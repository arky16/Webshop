use crate::services::products as product_service;
use crate::state::AppState;
use actix_web::{HttpResponse, web};
use uuid::Uuid;
use crate::models::product_request::{ProductQuery, ProductRequest};
use rust_decimal::Decimal;
use validator::Validate;
use redis::AsyncCommands;

async fn invalidate_product_list_cache(redis: &redis::Client) {
    let mut connection = match redis.get_multiplexed_async_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("Could not connect to Redis for cache invalidation: {error}");
            return;
        }
    };

    let mut cursor: u64 = 0;

    loop {
        let result: Result<(u64, Vec<String>), redis::RedisError> =
            redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg("products:list:*")
                .arg("COUNT")
                .arg(100)
                .query_async(&mut connection)
                .await;

        match result {
            Ok((next_cursor, keys)) => {
                cursor = next_cursor;

                if !keys.is_empty() {
                    let delete_result: Result<(), redis::RedisError> =
                        redis::cmd("DEL")
                            .arg(keys)
                            .query_async(&mut connection)
                            .await;

                    if let Err(error) = delete_result {
                        eprintln!("Could not delete product list cache: {error}");
                        return;
                    }
                }

                if cursor == 0 {
                    break;
                }
            }
            Err(error) => {
                eprintln!("Could not scan product cache: {error}");
                return;
            }
        }
    }

    println!("Product list cache invalidated");
}

async fn invalidate_product_item_cache(redis: &redis::Client, id: Uuid) {
    let cache_key = format!("products:item:{id}");

    match redis.get_multiplexed_async_connection().await {
        Ok(mut connection) => {
            let result: Result<(), redis::RedisError> =
                redis::cmd("DEL")
                    .arg(&cache_key)
                    .query_async(&mut connection)
                    .await;

            if let Err(error) = result {
                eprintln!("Could not delete product item cache: {error}");
            } else {
                println!("Product item cache invalidated: {id}");
            }
        }
        Err(error) => {
            eprintln!("Could not connect to Redis for item cache invalidation: {error}");
        }
    }
}


fn validate_product_request(request: &ProductRequest) -> bool{
    request.validate().is_ok() && request.price >= Decimal::ZERO && request.price.scale() <= 2
}

pub async fn list_products(state: web::Data<AppState>, query: web::Query<ProductQuery>) -> HttpResponse {
    let mut query = query.into_inner();

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

    query.page = Some(page);
    query.size = Some(size);
    query.sort_by = Some(sort_by.clone());
    query.order = Some(order.clone());

    let cache_key = match serde_json::to_string(&query) {
        Ok(parameters) => format!("products:list:{parameters}"),
        Err(error) => {
            eprintln!("Could not create product cache key: {error}");
            String::new()
        }
    };

    if !cache_key.is_empty() {
        match state.redis.get_multiplexed_async_connection().await {
            Ok(mut connection) => {
                let cached: Result<Option<String>, redis::RedisError> =
                    connection.get(&cache_key).await;

                match cached {
                    Ok(Some(cached_json)) => {
                        println!("Product list cache HIT");

                        return HttpResponse::Ok()
                            .content_type("application/json")
                            .body(cached_json);
                    }
                    Ok(None) => {
                        println!("Product list cache MISS");
                    }
                    Err(error) => {
                        eprintln!("Redis GET failed: {error}");
                    }
                }
            }
            Err(error) => {
                eprintln!("Could not connect to Redis: {error}");
            }
        }
    }

    match product_service::list_products(
        &state.db,
        &query,
        i64::from(page),
        i64::from(size),
        &sort_by,
        &order,
    )
        .await
    {
        Ok((products, total)) => {
            let total_pages = (total + i64::from(size) - 1) / i64::from(size);

            let response_body = serde_json::json!({
                "products": products,
                "pagination": {
                    "page": page,
                    "size": size,
                    "total": total,
                    "total_pages": total_pages
                }
            });

            if !cache_key.is_empty() {
                match serde_json::to_string(&response_body) {
                    Ok(cached_json) => {
                        match state.redis.get_multiplexed_async_connection().await {
                            Ok(mut connection) => {
                                let result: Result<(), redis::RedisError> =
                                    connection.set_ex(&cache_key, cached_json, 60).await;

                                if let Err(error) = result {
                                    eprintln!("Redis SET failed: {error}");
                                }
                            }
                            Err(error) => {
                                eprintln!("Could not connect to Redis: {error}");
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("Could not serialize product response: {error}");
                    }
                }
            }

            HttpResponse::Ok().json(response_body)
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
    let id = path.into_inner();

    let cache_key = format!("products:item:{id}");

    match state.redis.get_multiplexed_async_connection().await {
        Ok(mut connection) => {
            let cached: Result<Option<String>, redis::RedisError> =
                connection.get(&cache_key).await;

            match cached {
                Ok(Some(cached_json)) => {
                    println!("Product item cache HIT: {id}");

                    return HttpResponse::Ok()
                        .content_type("application/json")
                        .body(cached_json);
                }
                Ok(None) => {
                    println!("Product item cache MISS: {id}");
                }
                Err(error) => {
                    eprintln!("Redis GET failed: {error}");
                }
            }
        }
        Err(error) => {
            eprintln!("Could not connect to Redis: {error}");
        }
    }

    match product_service::get_product(&state.db, id).await {
        Ok(Some(product)) => {
            match serde_json::to_string(&product) {
                Ok(cached_json) => {
                    match state.redis.get_multiplexed_async_connection().await {
                        Ok(mut connection) => {
                            let result: Result<(), redis::RedisError> =
                                connection.set_ex(&cache_key, cached_json, 60).await;

                            if let Err(error) = result {
                                eprintln!("Redis SET failed: {error}");
                            }
                        }
                        Err(error) => {
                            eprintln!("Could not connect to Redis: {error}");
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Could not serialize product: {error}");
                }
            }

            HttpResponse::Ok().json(product)
        }

        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "PRODUCT_NOT_FOUND",
                "message": "Product not found"
            }))
        }

        Err(error) => {
            eprintln!("Could not fetch product: {error}");

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Product could not be loaded"
            }))
        }
    }
}

pub async fn create_product(state: web::Data<AppState>, body: web::Json<ProductRequest>) -> HttpResponse {
    if !validate_product_request(&body) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_PRODUCT",
            "message": "Check the product fields. Price must be non-negative and have at most two decimal places."
        }));
    }

    match product_service::create_product(&state.db, body.into_inner()).await {
        Ok(product) => {
            invalidate_product_list_cache(&state.redis).await;

            HttpResponse::Created().json(product)
        }
        Err(error) => {
            eprintln!("Could not create product: {error}");

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Product could not be created"
            }))
        }
    }
}

pub async fn update_product(state: web::Data<AppState>, path: web::Path<Uuid>, body: web::Json<ProductRequest>) -> HttpResponse {
    if !validate_product_request(&body) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_PRODUCT",
            "message": "Check the product fields. Price must be non-negative and have at most two decimal places."
        }));
    }

    let id = path.into_inner();

    match product_service::update_product(&state.db, id, body.into_inner()).await {
        Ok(Some(product)) => {
            invalidate_product_list_cache(&state.redis).await;
            invalidate_product_item_cache(&state.redis, id).await;

            HttpResponse::Ok().json(product)
        }

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
    let id = path.into_inner();

    match product_service::deactivate_product(&state.db, id).await {
        Ok(true) => {
            invalidate_product_list_cache(&state.redis).await;
            invalidate_product_item_cache(&state.redis, id).await;

            HttpResponse::NoContent().finish()
        }

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


