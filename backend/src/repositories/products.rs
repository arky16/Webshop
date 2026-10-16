use crate::models::product::Product;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_all(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"SELECT id, name, description, price, stock, category, active, created_at, updated_at
                FROM products
                WHERE active = TRUE
                ORDER BY created_at DESC
            "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
            SELECT  id, name, description, price, stock, category, active, created_at, updated_at
            FROM products
            WHERE id = $1 AND active = TRUE
            "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
