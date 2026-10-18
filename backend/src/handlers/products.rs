use crate::services::products as product_service;
use crate::state::AppState;
use actix_web::{HttpResponse, web};
use uuid::Uuid;
use crate::models::product_request::ProductRequest;
use rust_decimal::Decimal;
use validator::Validate;


fn validate_product_request(request: &ProductRequest) -> bool{
    request.validate().is_ok() && request.price >= Decimal::ZERO && request.price.scale() <= 2
}

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

pub async fn create_product(state: web::Data<AppState>, body: web::Json<ProductRequest>) -> HttpResponse{
    if !validate_product_request(&body) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_PRODUCT",
            "message": "Check the product fields. Price must be non-negative and have at most two decimal places."
        }));
    }

    match product_service::create_product(&state.db, body.into_inner()).await {
        Ok(product) => HttpResponse::Created().json(product),
        Err(error) => {
            eprintln!("Could not create product: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Product could not be created"
            }))
        }
    }
}

pub async fn update_product(state: web::Data<AppState>, path: web::Path<Uuid>, body: web::Json<ProductRequest>)-> HttpResponse{
    if !validate_product_request(&body) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_PRODUCT",
            "message": "Check the product fields. Price must be non-negative and have at most two decimal places."
        }));
    }

    let id = path.into_inner();

    match product_service::update_product(&state.db, id, body.into_inner()).await {
        Ok(Some(product)) => HttpResponse::Ok().json(product),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "PRODUCT_NOT_FOUND",
            "message": "Product not found"
        })),

        Err(error) => {
            eprintln!("Could not update product: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Product could not be updated"
            }))
        }
    }
}

pub async fn delete_product(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    match product_service::deactivate_product(&state.db, path.into_inner()).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "PRODUCT_NOT_FOUND",
            "message": "Product not found"
        })),

        Err(error) => {
            eprintln!("Could not deactivate product: {error}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Product could not be deleted"
            }))
        }
    }
}


