use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct User{
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String
}