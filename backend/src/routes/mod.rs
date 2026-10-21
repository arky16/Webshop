use actix_web::http::Method;
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
            .service(web::resource("")
                    .route(web::get()
                        .to(crate::handlers::products::list_products))
                    .route(web::route()
                        .method(Method::POST)
                        .to(crate::handlers::products::create_product)
                        .wrap(HttpAuthentication::bearer(crate::auth_middleware::admin_validator)))
            )
            .service(web::resource("/{id}")
                    .route(web::get()
                            .to(crate::handlers::products::get_product))
                    .route(web::route()
                            .method(Method::PUT)
                            .to(crate::handlers::products::update_product)
                            .wrap(HttpAuthentication::bearer(crate::auth_middleware::admin_validator)))
                    .route(web::route()
                            .method(Method::DELETE)
                            .to(crate::handlers::products::delete_product)
                            .wrap(HttpAuthentication::bearer(crate::auth_middleware::admin_validator)))
            )
    );
    
    cfg.service(
        web::scope("/api/account")
            .route("/me", web::get().to(crate::handlers::auth::me),)
            .wrap(HttpAuthentication::bearer(crate::auth_middleware::validator)),
    );
    
    cfg.service(
        web::scope("/api/cart")
            .route("", web::get().to(crate::handlers::cart::get_cart))
            .route("", web::post().to(crate::handlers::cart::add_to_cart))
            .route("/{product_id}", web::delete().to(crate::handlers::cart::remove_from_cart))
            .wrap(HttpAuthentication::bearer(crate::auth_middleware::validator))
    );

    cfg.service(
        web::scope("/api/orders")
            .route("/checkout", web::post().to(crate::handlers::orders::checkout))
            .route("", web::get().to(crate::handlers::orders::get_my_orders))
            .wrap(HttpAuthentication::bearer(crate::auth_middleware::validator))
    );
    
    cfg.service(
        web::scope("/api/admin")
            .route("/orders", web::get().to(crate::handlers::orders::get_all_orders))
            .wrap(HttpAuthentication::bearer(crate::auth_middleware::admin_validator))
    );
    
    
}
