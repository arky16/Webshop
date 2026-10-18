use crate::models::product::Product;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::product_request::ProductRequest;

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

pub async fn create(pool: &PgPool, request: ProductRequest) -> Result<Product, sqlx::Error>{
    sqlx::query_as::<_, Product>(
        r#"
            INSERT INTO products(
                                 id, name, description, price, stock, category
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, description, price, stock, category, active, created_at, updated_at
            "#,
    )
        .bind(Uuid::new_v4())
        .bind(request.name)
        .bind(request.description)
        .bind(request.price)
        .bind(request.stock)
        .bind(request.category)
        .fetch_one(pool)
        .await
}


pub async fn update(pool: &PgPool, id: Uuid, request: ProductRequest) -> Result<Option<Product>, sqlx::Error>{
    sqlx::query_as::<_, Product>(
        r#"
            UPDATE products
            SET name = $2,
                description = $3,
                price = $4,
                stock = $5,
                category = $6,
                updated_at = NOW()
            WHERE id = $1 AND active = TRUE
            RETURNING id, name, description, price, stock, category, active, created_at, updated_at
            "#,
    )
        .bind(id)
        .bind(request.name)
        .bind(request.description)
        .bind(request.price)
        .bind(request.stock)
        .bind(request.category)
        .fetch_optional(pool)
        .await
}

pub async fn deactivate(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error>{
    let result = sqlx::query(
        r#"
            UPDATE products
            SET active = FALSE,
                updated_at = NOW()
            WHERE id = $1 AND active = TRUE
            "#,
    )
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}