use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;
use crate::services::orders::{self, CheckoutError};
use crate::models::claims::Claims;
use crate::state::AppState;

pub async fn checkout(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse{
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "UNAUTHORIZED",
                "message": "Authentication required"
            }))
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "INVALID_TOKEN",
                "message": "Invalid user identifier in token"
            }))
        }
    };

    match orders::checkout(&state.db, user_id).await {
        Ok((order, items)) => {
            HttpResponse::Created().json(serde_json::json!({
                "order": order,
                "items": items
            }))
        }
        Err(CheckoutError::EmptyCart) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "EMPTY_CART",
                "message": "Cannot checkout with an empty cart"
            }))
        }
        Err(CheckoutError::InsufficientStock) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "INSUFFICIENT_STOCK",
                "message": "A product is unavailable or stock is insufficient"
            }))
        }
        Err(CheckoutError::Database(error)) => {
            eprintln!("Checkout failed: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Checkout could not be completed"
            }))
        }
    }
}