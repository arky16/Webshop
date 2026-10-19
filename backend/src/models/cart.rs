use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct CartItem{
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Serialize, FromRow)]
pub struct CartItemDetails{
    pub id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub description: String,
    pub price: Decimal,
    pub stock: i32,
    pub quantity: i32,
    pub subtotal: Decimal
}