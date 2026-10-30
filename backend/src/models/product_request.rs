use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ProductRequest{
    #[validate(length(min=1, max=200))]
    pub name: String,
    
    #[validate(length(max=5000))]
    pub description: String,
    
    pub price: Decimal,
    
    #[validate(range(min=0))]
    pub stock: i32,
    
    #[validate(length(max=100))]
    pub category: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductQuery{
    pub search: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub page: Option<u32>,
    pub size: Option<u32>
}