use crate::models::product::Product;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::product_request::{ProductRequest, ProductQuery};

pub async fn find_all(pool: &PgPool, query: &ProductQuery, page: i64, size: i64, sort_by: &str, order: &str) -> Result<(Vec<Product>, i64), sqlx::Error> {
    let offset = page * size;

    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let category = query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let min_price = query.min_price;
    let max_price = query.max_price;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM products
        WHERE active = TRUE
          AND ($1::text IS NULL OR name ILIKE $1)
          AND ($2::text IS NULL OR category = $2)
          AND ($3::numeric IS NULL OR price >= $3)
          AND ($4::numeric IS NULL OR price <= $4)
        "#
    )
        .bind(&search)
        .bind(category)
        .bind(min_price)
        .bind(max_price)
        .fetch_one(pool)
        .await?;

    let sort_column = match sort_by {
        "price" => "price",
        "name" => "name",
        _ => "created_at",
    };

    let sort_direction = match order {
        "asc" => "ASC",
        _ => "DESC",
    };

    let sql = format!(
        r#"
        SELECT id, name, description, price, stock, category, active, created_at, updated_at
        FROM products
        WHERE active = TRUE
          AND ($1::text IS NULL OR name ILIKE $1)
          AND ($2::text IS NULL OR category = $2)
          AND ($3::numeric IS NULL OR price >= $3)
          AND ($4::numeric IS NULL OR price <= $4)
        ORDER BY {sort_column} {sort_direction}, id ASC
        LIMIT $5 OFFSET $6
        "#
    );

    let products = sqlx::query_as::<_, Product>(&sql)
        .bind(&search)
        .bind(category)
        .bind(min_price)
        .bind(max_price)
        .bind(size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((products, total))
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