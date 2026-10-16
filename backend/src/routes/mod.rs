use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/products")
            .service(crate::handlers::products::list_products)
            .service(crate::handlers::products::get_product),
    );
}
