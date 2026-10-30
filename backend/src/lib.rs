pub mod handlers;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod auth_middleware;

use actix_web::{web, App};
use state::AppState;

pub fn create_app(state: AppState) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(state))
        .configure(routes::configure)
}
