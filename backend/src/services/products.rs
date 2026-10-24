use crate::models::product::Product;
use crate::repositories::products as product_repository;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::product_request::{ProductRequest, ProductQuery};

pub async fn list_products(pool: &PgPool, query: &ProductQuery, page: i64, size: i64, sort_by: &str, order: &str) -> Result<(Vec<Product>, i64), sqlx::Error> {
    product_repository::find_all(pool, query, page, size, sort_by, order).await
}

pub async fn get_product(pool: &PgPool, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
    product_repository::find_by_id(pool, id).await
}

pub async fn create_product(pool: &PgPool, request: ProductRequest) -> Result<Product, sqlx::Error>{
    product_repository::create(pool, request).await
}

pub async fn update_product(pool: &PgPool, id: Uuid, request: ProductRequest) -> Result<Option<Product>, sqlx::Error>{
    product_repository::update(pool, id, request).await
}

pub async fn deactivate_product(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error>{
    product_repository::deactivate(pool, id).await
}