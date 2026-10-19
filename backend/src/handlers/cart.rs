use crate::models::claims::Claims;
use crate::services::cart::{self, CartError};
use crate::state::AppState;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AddCartItemRequest{
    pub product_id: Uuid,
    pub quantity: i32
}

fn authenticated_user_id(req: &HttpRequest) -> Result<Uuid, HttpResponse>{
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "UNAUTHORIZED",
                "message": "Authentication required"
            }))
        })?;

    Uuid::parse_str(&claims.sub).map_err(|_| {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "INVALID_TOKEN",
            "message": "Invalid user identifier in token"
        }))
    })
}

pub async fn get_cart(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse{
    let user_id = match authenticated_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response
    };

    match cart::get_cart(&state.db, user_id).await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(CartError::Database(error)) => {
            eprintln!("Could not load cart: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Cart could not be loaded"
            }))
        }
        _ => HttpResponse::InternalServerError().finish()
    }
}

pub async fn add_to_cart(req: HttpRequest, state: web::Data<AppState>, body: web::Json<AddCartItemRequest>) -> HttpResponse{
    let user_id = match authenticated_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response
    };

    if body.quantity <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_QUANTITY",
            "message": "Quantity must be greater then zero"
        }));
    }

    match cart::add_to_cart(&state.db, user_id, body.product_id, body.quantity).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(CartError::InvalidQuantity) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "INVALID_QUANTITY",
                "message": "Quantity must be greater then zero"
            }))
        }
        Err(CartError::ProductUnavailable) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "PRODUCT_UNAVAILABLE",
                "message": "Product is unavailable or stock is insufficient"
            }))
        }
        Err(CartError::Database(error)) => {
            eprintln!("Could not update cart: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Cart could not be updated"
            }))
        }
    }
}

pub async fn remove_from_cart(req: HttpRequest, state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse{
    let user_id = match authenticated_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response
    };

    match cart::remove_from_cart(&state.db, user_id, path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(CartError::ProductUnavailable) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "CART_ITEM_NOT_FOUND",
                "message":"Product was not found in this user's cart"
            }))
        }
        Err(CartError::Database(error)) => {
            eprintln!("Could not remove cart item: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Cart item could not be removed"
            }))
        }
        Err(CartError::InvalidQuantity) => {
            HttpResponse::BadRequest().finish()
        }
    }
}