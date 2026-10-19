use crate::models::cart::CartItemDetails;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_all(pool: &PgPool, user_id: Uuid) -> Result<Vec<CartItemDetails>, sqlx::Error>{
    sqlx::query_as::<_, CartItemDetails>(
        r#"
            SELECT 
                c.id,
                c.product_id,
                p.name,
                p.description,
                p.price,
                p.stock,
                c.quantity,
                (p.price * c.quantity)::NUMERIC(12,2) AS subtotal
            FROM cart_items c
            JOIN products p ON p.id = c.product_id
            WHERE c.user_id = $1 AND p.active = TRUE
            ORDER BY c.created_at DESC 
            "#
    )
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn add_item(pool: &PgPool, user_id: Uuid, product_id: Uuid, quantity: i32) -> Result<Option<CartItemDetails>, sqlx::Error>{
    sqlx::query_as::<_, CartItemDetails>(
        r#"
            WITH upserted AS ( 
                INSERT INTO cart_items(id, user_id, product_id, quantity)
                SELECT $1, p_user.id, p.id, $4
                FROM products p 
                JOIN users p_user ON p_user.id =$2
                WHERE p.id =$3 AND p.active = TRUE AND p.stock >= $4
                ON CONFLICT (user_id, product_id)
                DO UPDATE SET quantity = cart_items.quantity + EXCLUDED.quantity, updated_at = NOW()
                WHERE cart_items.quantity + EXCLUDED.quantity <= (
                    SELECT stock
                    FROM products
                    WHERE id = EXCLUDED.product_id AND active = TRUE
                )
                RETURNING id ,user_id, product_id, quantity, created_at, updated_at
             )
            SELECT 
                c.id,
                c.product_id,
                p.name,
                p.description,
                p.price,
                p.stock,
                c.quantity,
                (p.price * c.quantity)::NUMERIC(12,2) AS subtotal
            FROM upserted c
            JOIN products p ON p.id = c.product_id
            "#
    )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(product_id)
        .bind(quantity)
        .fetch_optional(pool)
        .await
}

pub async fn remove_item(pool: &PgPool, user_id: Uuid, product_id: Uuid) -> Result<bool, sqlx::Error>{
    let result = sqlx::query(
        r#"
            DELETE FROM cart_items
            WHERE user_id = $1 AND product_id = $2
            "#
    )
        .bind(user_id)
        .bind(product_id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}