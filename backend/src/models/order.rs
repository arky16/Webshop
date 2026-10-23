use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub total_amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Serialize, FromRow)]
pub struct OrderItem{
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest{
    pub status: String
}

#[derive(Debug, Deserialize)]
pub struct AdminOrdersQuery{
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub status: Option<String>
}