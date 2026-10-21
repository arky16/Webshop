use crate::models::order::{Order, OrderItem};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum CheckoutError{
    EmptyCart,
    InsufficientStock,
    Database(sqlx::Error)
}

pub async fn checkout(pool: &PgPool, user_id: Uuid) -> Result<(Order, Vec<OrderItem>), CheckoutError>{
    let mut tx = pool.begin().await.map_err(CheckoutError::Database)?;

    let cart_items = sqlx::query_as::<_, (Uuid, i32)>(
        r#"
            SELECT product_id, quantity
            FROM cart_items
            WHERE user_id = $1
            ORDER BY product_id
            FOR UPDATE
            "#,
    )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(CheckoutError::Database)?;

    if cart_items.is_empty(){
        return Err(CheckoutError::EmptyCart);
    }

    let order_id = Uuid::new_v4();
    let mut total = Decimal::ZERO;
    let mut order_items = Vec::new();

    for (product_id, quantity) in cart_items{
        let product = sqlx::query_as::<_, (Decimal, i32, bool)>(
            r#"
                SELECT price, stock, active
                FROM products
                WHERE id = $1
                FOR UPDATE
                "#,
        )
            .bind(product_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(CheckoutError::Database)?;

        let (price, stock, active) = product.ok_or(CheckoutError::InsufficientStock)?;

        if !active || stock < quantity {
            return Err(CheckoutError::InsufficientStock);
        }

        let updated = sqlx::query(
            r#"
                UPDATE products
                SET stock = stock - $1,
                    updated_at = NOW()
                WHERE id = $2 AND active = TRUE AND stock >= $1
                "#,
        )
            .bind(quantity)
            .bind(product_id)
            .execute(&mut *tx)
            .await
            .map_err(CheckoutError::Database)?;

        if updated.rows_affected() != 1 {
            return Err(CheckoutError::InsufficientStock)
        }

        total += price * Decimal::from(quantity);

        order_items.push(OrderItem{
            id: Uuid::new_v4(),
            order_id,
            product_id,
            quantity,
            unit_price: price,
        });
    }

    let order = sqlx::query_as::<_, Order>(
        r#"
            INSERT INTO orders (id, user_id, status, total_amount)
            VALUES ($1, $2, 'PENDING', $3)
            RETURNING id, user_id, status, total_amount, created_at, updated_at
            "#,
    )
        .bind(order_id)
        .bind(user_id)
        .bind(total)
        .fetch_one(&mut *tx)
        .await
        .map_err(CheckoutError::Database)?;

    for item in &order_items{
        sqlx::query(
            r#"
                INSERT INTO order_items (id, order_id, product_id, quantity, unit_price)
                VALUES ($1, $2, $3, $4, $5)
                "#,
        )
            .bind(item.id)
            .bind(item.order_id)
            .bind(item.product_id)
            .bind(item.quantity)
            .bind(item.unit_price)
            .execute(&mut *tx)
            .await
            .map_err(CheckoutError::Database)?;
    }

    sqlx::query("DELETE FROM cart_items WHERE user_id =$1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(CheckoutError::Database)?;

    tx.commit().await.map_err(CheckoutError::Database)?;

    Ok((order, order_items))
}

pub async fn find_orders_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Order>, sqlx::Error>{
    sqlx::query_as::<_, Order>(
        r#"
            SELECT id, user_id, status, total_amount, created_at, updated_at
            FROM orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#
    )
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn find_order_items(pool: &PgPool, order_id: Uuid) -> Result<Vec<OrderItem>, sqlx::Error>{
    sqlx::query_as::<_, OrderItem>(
        r#"
            SELECT id, order_id, product_id, quantity, unit_price
            FROM order_items
            WHERE order_id = $1
            ORDER BY id
            "#
    )
        .bind(order_id)
        .fetch_all(pool)
        .await
}

pub async fn find_all_orders(pool: &PgPool) -> Result<Vec<Order>, sqlx::Error>{
    sqlx::query_as::<_, Order>(
        r#"
            SELECT id, user_id, status, total_amount, created_at, updated_at
            FROM orders
            ORDER BY created_at DESC
            "#
    )
        .fetch_all(pool)
        .await
}