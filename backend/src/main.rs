use backend::state::AppState;
use backend::routes;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use actix_cors;
use actix_cors::Cors;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    database: &'static str,
    redis: &'static str
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Webshop API is running!")
}

#[get("/health")]
async fn health(state: web::Data<AppState>) -> impl Responder {
    let database_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    let redis_ok = match state.redis.get_multiplexed_async_connection().await {
        Ok(mut connection) => {
            matches!(
                redis::cmd("PING")
                    .query_async::<String>(&mut connection)
                    .await,
                Ok(response) if response == "PONG"
            )
        }
        Err(_) => false,
    };

    if database_ok && redis_ok {
        HttpResponse::Ok().json(HealthResponse {
            status: "ok",
            service: "webshop-api",
            database: "connected",
            redis: "connected",
        })
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "error",
            "service": "webshop-api",
            "database": if database_ok { "connected" } else { "unavailable" },
            "redis": if redis_ok { "connected" } else { "unavailable" }
        }))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be configured");

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Could not connect to PostgreSQL");

    println!("Connected to PostgreSQL.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Database migrations failed");

    println!("Database migrations completed.");

    let redis_url = std::env::var("REDIS_URL")
        .expect("REDIS_URL must be configured");

    let redis_client = redis::Client::open(redis_url)
        .expect("Could not create Redis client");

    let redis_connection = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Could not connect to Redis");

    println!("Connected to Redis.");

    drop(redis_connection);

    let state = web::Data::new(AppState { db: pool, redis: redis_client});

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
                .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE, actix_web::http::header::AUTHORIZATION])
                .max_age(3600)
            )
            .app_data(state.clone())
            .service(index)
            .service(health)
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}

