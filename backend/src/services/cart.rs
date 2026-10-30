use crate::models::cart::CartItemDetails;
use crate::repositories::cart as cart_repository;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum CartError{
    InvalidQuantity,
    ProductUnavailable,
    Database(sqlx::Error)
}

fn validate_quantity(quantity: i32) -> Result<(), CartError> {
    if quantity <= 0 {
        return Err(CartError::InvalidQuantity);
    }

    Ok(())
}

pub async fn get_cart(pool: &PgPool, user_id: Uuid) -> Result<Vec<CartItemDetails>, CartError>{
    cart_repository::find_all(pool, user_id)
        .await
        .map_err(CartError::Database)
}

pub async fn add_to_cart(pool: &PgPool, user_id: Uuid, product_id: Uuid, quantity: i32) -> Result<CartItemDetails, CartError>{
    validate_quantity(quantity)?;

    match cart_repository::add_item(pool, user_id, product_id, quantity)
        .await
        .map_err(CartError::Database)? {
        Some(item) => Ok(item),
        None => Err(CartError::ProductUnavailable)
    }
}

pub async fn remove_from_cart(pool: &PgPool, user_id: Uuid, product_id: Uuid) -> Result<(), CartError>{
    let removed = cart_repository::remove_item(pool, user_id, product_id)
        .await
        .map_err(CartError::Database)?;

    if removed {
        Ok(())
    } else {
        Err(CartError::ProductUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::{CartError, validate_quantity};

    #[test]
    fn positive_quantity_is_valid() {
        assert!(validate_quantity(1).is_ok());
        assert!(validate_quantity(5).is_ok());
        assert!(validate_quantity(100).is_ok());
    }

    #[test]
    fn zero_quantity_is_invalid() {
        assert!(matches!(
            validate_quantity(0),
            Err(CartError::InvalidQuantity)
        ));
    }

    #[test]
    fn negative_quantity_is_invalid() {
        assert!(matches!(
            validate_quantity(-1),
            Err(CartError::InvalidQuantity)
        ));

        assert!(matches!(
            validate_quantity(-10),
            Err(CartError::InvalidQuantity)
        ));
    }
}