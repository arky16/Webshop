use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;
use crate::services::orders::{self, CheckoutError};
use crate::models::claims::Claims;
use crate::state::AppState;
use crate::models::order::UpdateOrderStatusRequest;
use crate::repositories::orders::UpdateOrderStatusError;

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

pub async fn get_my_orders(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse{
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "UNAUTHORIZED",
                "message": "Authentication required"
            }));
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "INVALID_TOKEN",
                "message": "Invalid user identifier in token"
            }));
        }
    };

    match orders::get_orders_by_user(&state.db, user_id).await {
        Ok(orders) => {
            let responce: Vec<_> = orders
                .into_iter()
                .map(|(order, items)| {
                    serde_json::json!({
                        "order": order,
                        "items": items
                    })
                })
                .collect();

            HttpResponse::Ok().json(responce)
        }
        Err(error) => {
            eprintln!("Could not load orders: {error}");

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Orders could not be loaded"
            }))
        }
    }
}


pub async fn get_all_orders(state: web::Data<AppState>, query: web::Query<crate::models::order::AdminOrdersQuery>) -> Result<HttpResponse, actix_web::Error> {
    let query = query.into_inner();

    let page = query.page.unwrap_or(0);
    let size = query.size.unwrap_or(10);

    if size == 0 || size > 100 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Size must be between 1 and 100"
            })));
    }

    let status = query.status.map(|s| s.trim().to_uppercase());

    if let Some(ref s) = status {
        if !matches!(
            s.as_str(),
            "PENDING" | "PAID" | "PROCESSING" | "SHIPPED" | "COMPLETED" | "CANCELLED"
        ) {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Invalid order status filter"
                })));
        }
    }

    let page = i64::from(page);
    let size = i64::from(size);

    let status_ref = status.as_deref();

    match orders::get_all_orders(&state.db, page, size, status_ref).await {
        Ok((orders, total)) => {
            let order_ids: Vec<uuid::Uuid> = orders.iter().map(|order| order.id).collect();

            let items = match crate::repositories::orders::find_order_items_for_orders(&state.db, &order_ids).await {
                Ok(items) => items,
                Err(err) => {
                    eprintln!("Failed to fetch order items: {err}");

                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "message": "Failed to fetch order items"
                    })));
                }
            };

            let mut response = Vec::with_capacity(orders.len());

            for order in orders {
                let order_items: Vec<_> = items
                    .iter()
                    .filter(|item| item.order_id == order.id)
                    .collect();

                response.push(serde_json::json!({
                    "order": order,
                    "items": order_items
                }));
            }

            let total_pages = (total + size - 1) / size;

            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "orders": response,
                "pagination": {
                    "page": page,
                    "size": size,
                    "total": total,
                    "total_pages": total_pages
                }
            })));
        }

        Err(err) => {
            eprintln!("Failed to fetch orders: {err}");

            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "message": "Failed to fetch orders"
            })))
        }
    }
}


pub async fn update_order_status(state: web::Data<AppState>, path: web::Path<Uuid>, body: web::Json<UpdateOrderStatusRequest>) -> Result<HttpResponse, actix_web::Error>{
    let order_id = path.into_inner();
    let status = body.status.trim().to_uppercase();
    
    if !matches!(
        status.as_str(),
        "PROCESSING" | "SHIPPED" | "COMPLETED" | "CANCELLED"
    ){
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Invalid order status"
        })));
    }
    
    match orders::change_order_status(&state.db, order_id, &status).await { 
        Ok(order) => {Ok(HttpResponse::Ok().json(order))}
        
        Err(UpdateOrderStatusError::NotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "message": "Order not found"
            })))
        }
        
        Err(UpdateOrderStatusError::InvalidTransition) => {
            Ok(HttpResponse::Conflict().json(serde_json::json!({
                "message": "Invalid order status transition"
            })))
        }
        
        Err(UpdateOrderStatusError::Database(err)) => {
            eprintln!("Failed to update order status: {err}");
            
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Failed to update order status"
            })))
        }
    }
}

pub async fn get_order_details(req: HttpRequest, state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse{
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

    let order_id = path.into_inner();

    let order = match orders::find_order_by_user(&state.db, order_id, user_id).await {
        Ok(Some(order)) => order,

        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "ORDER_NOT_FOUND",
                "message": "Order not found"
            }))
        }

        Err(err) => {
            eprintln!("Failed to fetch order: {err}");

            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Order could not be loaded"
            }))
        }
    };

    let items = match crate::repositories::orders::find_order_items(&state.db, order.id).await {
        Ok(items) => items,

        Err(err) => {
            eprintln!("Failed to fetch order items: {err}");

            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Order items could not be loaded"
            }))
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "order": order,
        "items": items
    }))
}