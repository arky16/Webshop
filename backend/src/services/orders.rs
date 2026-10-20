use sqlx::PgPool;
use uuid::Uuid;
use crate::models::order::{Order, OrderItem};
pub(crate) use crate::repositories::orders::{self, CheckoutError};

pub async fn checkout(pool: &PgPool, user_id: Uuid) -> Result<(Order, Vec<OrderItem>), CheckoutError>{
    orders::checkout(pool, user_id).await
}