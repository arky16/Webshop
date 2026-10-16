use crate::services::products as product_service;
use crate::state::AppState;
use actix_web::{HttpResponse, get, web};
use uuid::Uuid;

#[get("")]
pub async fn list_products(state: web::Data<AppState>) -> HttpResponse {
    match product_service::list_products(&state.db).await {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(error) => {
            eprintln!("Could not fetch products: {error}");

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error":"INTERNAL_SERVER_ERROR",
                "message": "Products could not be loaded"
            }))
        }
    }
}

#[get("/{id}")]
pub async fn get_product(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    match product_service::get_product(&state.db, path.into_inner()).await {
        Ok(Some(product)) => HttpResponse::Ok().json(product),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "PRODUCT_NOT_FOUND",
            "message": "Product not found"
        })),
        Err(error) => {
            eprintln!("Could not fetch product: {error}");

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Product could not be loaded"
            }))
        }
    }
}
