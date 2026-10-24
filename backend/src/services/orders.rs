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

pub async fn get_all_orders(pool: &PgPool, page: i64, size: i64, status: Option<&str>) -> Result<(Vec<Order>, i64), sqlx::Error>{
    orders::find_all_orders(pool, page, size, status).await
}

pub async fn change_order_status(pool: &PgPool, order_id: Uuid, new_status: &str) -> Result<Order, orders::UpdateOrderStatusError>{
    orders::update_order_status(pool, order_id, new_status).await
}

pub async fn find_order_by_user(pool: &PgPool, order_id: Uuid, user_id: Uuid) -> Result<Option<Order>, sqlx::Error>{
    orders::find_order_by_user(pool, order_id, user_id).await
}