use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .service(crate::handlers::auth::register)
            .service(crate::handlers::auth::login)
    );

    cfg.service(
        web::scope("/api/products")
            .service(crate::handlers::products::list_products)
            .service(crate::handlers::products::get_product),
    );
    
    cfg.service(
        web::scope("/api/account")
            .route("/me", web::get().to(crate::handlers::auth::me),)
            .wrap(HttpAuthentication::bearer(crate::auth_middleware::validator)),
    );
}
