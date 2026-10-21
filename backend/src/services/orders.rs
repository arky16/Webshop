use sqlx::PgPool;
use uuid::Uuid;
use crate::models::order::{Order, OrderItem};
pub(crate) use crate::repositories::orders::{self, CheckoutError};

pub async fn checkout(pool: &PgPool, user_id: Uuid) -> Result<(Order, Vec<OrderItem>), CheckoutError>{
    orders::checkout(pool, user_id).await
}

pub async fn get_orders_by_user(pool: &PgPool, user_id: Uuid)->Result<Vec<(Order, Vec<OrderItem>)>, sqlx::Error>{
    let orders = orders::find_orders_by_user(pool, user_id).await?;

    let mut result = Vec::with_capacity(orders.len());

    for order in orders {
        let items = orders::find_order_items(pool, order.id).await?;

        result.push((order, items));
    }

    Ok(result)
}

pub async fn get_all_orders(pool: &PgPool) -> Result<Vec<(Order, Vec<OrderItem>)>, sqlx::Error>{
    let orders = orders::find_all_orders(pool).await?;

    let mut result = Vec::with_capacity(orders.len());

    for order in orders {
        let items = orders::find_order_items(pool, order.id).await?;
        result.push((order, items));
    }

    Ok(result)
}