use sqlx::PgPool;
use crate::models::user::User;
use crate::repositories::auth as auth_repository;
use argon2::{
    Argon2,
    password_hash::{
        PasswordHasher, SaltString, rand_core::OsRng,
    },
};

#[derive(Debug)]
pub enum RegisterError{
    EmailAlreadyExists,
    PasswordHashing,
    Database(sqlx::Error)
}

pub async fn register_user(pool: &PgPool, email: &str, password: &str) -> Result<User, RegisterError>{

    let email = email.trim().to_lowercase();

    let existing_user = auth_repository::find_by_email(pool, &email).await.map_err(RegisterError::Database)?;

    if existing_user.is_some() {
        return Err(RegisterError::EmailAlreadyExists);
    }

    let salt = SaltString::generate(&mut OsRng);

    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| RegisterError::PasswordHashing)?
        .to_string();

    match auth_repository::create_user(pool, &email, &password_hash).await {
        Ok(user) => Ok(user),
        Err(error) => {
            let is_duplicate = error
                .as_database_error()
                .and_then(|db_error| db_error.code())
                .as_deref() == Some("23505");

            if is_duplicate{
                Err(RegisterError::EmailAlreadyExists)
            }else {
                Err(RegisterError::Database(error))
            }
        }
    }
}

