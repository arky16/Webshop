use crate::models::product::Product;
use crate::repositories::products as product_repository;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    product_repository::find_all(pool).await
}

pub async fn get_product(pool: &PgPool, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
    product_repository::find_by_id(pool, id).await
}
