use crate::services::products as product_service;
use crate::state::AppState;
use actix_web::{HttpResponse, web};
use uuid::Uuid;
use crate::models::product_request::{ProductQuery, ProductRequest};
use rust_decimal::Decimal;
use validator::Validate;


fn validate_product_request(request: &ProductRequest) -> bool{
    request.validate().is_ok() && request.price >= Decimal::ZERO && request.price.scale() <= 2
}

pub async fn list_products(state: web::Data<AppState>, query: web::Query<ProductQuery>) -> HttpResponse {
    let query = query.into_inner();

    let page = query.page.unwrap_or(0);
    let size = query.size.unwrap_or(10);

    if size == 0 || size > 100 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_PAGINATION",
            "message": "Size must be between 1 and 100"
        }));
    }

    let sort_by = query
        .sort_by
        .as_deref()
        .unwrap_or("created_at")
        .to_lowercase();

    if !["name", "price", "created_at"].contains(&sort_by.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_SORT_FIELD",
            "message": "sort_by must be name, price or created_at"
        }));
    }

    let order = query
        .order
        .as_deref()
        .unwrap_or("desc")
        .to_lowercase();

    if !["asc", "desc"].contains(&order.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_SORT_ORDER",
            "message": "order must be asc or desc"
        }));
    }

    if query.min_price.is_some_and(|price| price < Decimal::ZERO)
        || query.max_price.is_some_and(|price| price < Decimal::ZERO)
        || matches!(
            (query.min_price, query.max_price),
            (Some(min), Some(max)) if min > max
        )
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_PRICE_RANGE",
            "message": "Prices must be non-negative and min_price must not exceed max_price"
        }));
    }

    match product_service::list_products(&state.db, &query, i64::from(page), i64::from(size), &sort_by, &order).await {
        Ok((products, total)) => {
            let total_pages = (total + i64::from(size) - 1) / i64::from(size);

            HttpResponse::Ok().json(serde_json::json!({
                "products": products,
                "pagination": {
                    "page": page,
                    "size": size,
                    "total": total,
                    "total_pages": total_pages
                }
            }))
        }
        Err(error) => {
            eprintln!("Could not fetch products: {error}");

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
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


