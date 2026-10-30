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

fn hash_password(password: &str) -> Result<String, ()> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| ())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

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

    let password_hash = hash_password(password)
        .map_err(|_| RegisterError::PasswordHashing)?;

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

    if !verify_password(password, &user.password_hash) {
        return Err(LoginError::InvalidCredentials);
    }

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

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};

    #[test]
    fn password_is_hashed_successfully() {
        let password = "MySecurePassword123";

        let hash = hash_password(password).unwrap();

        assert_ne!(hash, password);
    }

    #[test]
    fn correct_password_is_verified() {
        let password = "MySecurePassword123";

        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash));
    }

    #[test]
    fn incorrect_password_is_rejected() {
        let password = "MySecurePassword123";

        let hash = hash_password(password).unwrap();

        assert!(!verify_password("WrongPassword123", &hash));
    }

    #[test]
    fn invalid_hash_is_rejected() {
        assert!(!verify_password("password123", "invalid-hash"));
    }

    #[test]
    fn different_hashes_are_generated_for_same_password() {
        let password = "MySecurePassword123";

        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        assert_ne!(hash1, hash2);

        assert!(verify_password(password, &hash1));
        assert!(verify_password(password, &hash2));
    }
}



