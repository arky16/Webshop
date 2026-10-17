use sqlx::PgPool;
use crate::models::user::User;
use crate::repositories::auth as auth_repository;
use argon2::{
    Argon2,
    password_hash::{
        PasswordHasher, SaltString, rand_core::OsRng,
    },
};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::claims::Claims;

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

#[derive(Debug)]
pub enum LoginError{
    InvalidCredentials,
    TokenCreation,
    Configuration,
}

pub async fn login_user(pool: &PgPool, email: &str, password: &str)->Result<(User, String), LoginError>{
    let email = email.trim().to_lowercase();

    let user = auth_repository::find_by_email(pool, &email)
        .await
        .map_err(|_| LoginError::InvalidCredentials)?
        .ok_or(LoginError::InvalidCredentials)?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| LoginError::InvalidCredentials)?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| LoginError::InvalidCredentials)?;

    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| LoginError::Configuration)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| LoginError::TokenCreation)?
        .as_secs();

    let claims = Claims{
        sub: user.id.to_string(),
        role: user.role.clone(),
        exp: now + 3600,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(|_| LoginError::TokenCreation)?;

    Ok((user, token))
}



