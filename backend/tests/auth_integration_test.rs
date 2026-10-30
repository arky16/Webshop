use actix_web::{test, web, App};
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

use backend::{
    routes,
    state::AppState,
};

#[actix_web::test]
async fn auth_register_and_login_integration_test() {
    dotenvy::dotenv().ok();

    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Could not connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations failed");

    let email = format!("auth_test_{}@example.com", Uuid::new_v4());
    let password = "TestPassword123!";

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .configure(routes::configure),
    )
        .await;

    let register_request = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(serde_json::json!({
            "email": email,
            "password": password
        }))
        .to_request();

    let register_response = test::call_service(&app, register_request).await;

    assert_eq!(
        register_response.status(),
        actix_web::http::StatusCode::CREATED
    );

    let login_request = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "email": email,
            "password": password
        }))
        .to_request();

    let login_response = test::call_service(&app, login_request).await;

    assert_eq!(
        login_response.status(),
        actix_web::http::StatusCode::OK
    );

    let login_body: serde_json::Value =
        test::read_body_json(login_response).await;

    let token = login_body["access_token"]
        .as_str()
        .expect("Access token missing")
        .to_string();

    assert_eq!(login_body["token_type"], "Bearer");
    assert_eq!(login_body["expires_in"], 3600);

    let me_request = test::TestRequest::get()
        .uri("/api/account/me")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();

    let me_response = test::call_service(&app, me_request).await;

    assert_eq!(
        me_response.status(),
        actix_web::http::StatusCode::OK
    );

    let me_body: serde_json::Value =
        test::read_body_json(me_response).await;

    assert_eq!(me_body["id"], login_body["user"]["id"]);
    assert_eq!(me_body["role"], login_body["user"]["role"]);

    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(&pool)
        .await
        .expect("Could not clean up test user");

    pool.close().await;
}


#[actix_web::test]
async fn auth_error_cases_integration_test() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Could not connect to test database");

    let email = format!("auth_error_{}@example.com", Uuid::new_v4());
    let password = "TestPassword123!";

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .configure(routes::configure),
    )
        .await;

    let request = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(serde_json::json!({
            "email": email,
            "password": password
        }))
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::CREATED
    );

    let request = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(serde_json::json!({
            "email": email,
            "password": password
        }))
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::CONFLICT
    );

    let request = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(serde_json::json!({
            "email": "invalid-email",
            "password": "123"
        }))
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );

    let request = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "email": email,
            "password": "WrongPassword123!"
        }))
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::UNAUTHORIZED
    );

    let request = test::TestRequest::get()
        .uri("/api/account/me")
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::UNAUTHORIZED
    );

    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(&pool)
        .await
        .expect("Could not clean up test user");

    pool.close().await;
}


