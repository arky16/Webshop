use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::User;

pub async fn find_by_email(pool: &PgPool, email:&str)->Result<Option<User>, sqlx::Error>{
    sqlx::query_as::<_, User>(
        r#"
            SELECT id, email, password_hash, role
            FROM users
            WHERE email = $1
            "#,
    )
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(pool: &PgPool, email: &str, password_hash: &str)-> Result<User, sqlx::Error>{
    sqlx::query_as::<_, User>(
        r#"
            INSERT INTO users (id, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id ,email, password_hash, role
            "#,
    )
        .bind(Uuid::new_v4())
        .bind(email).bind(password_hash)
        .fetch_one(pool)
        .await
}