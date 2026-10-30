use crate::models::claims::Claims;
use actix_web::{
    dev::ServiceRequest,
    error::{ErrorInternalServerError, ErrorUnauthorized, ErrorForbidden},
    Error
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::HttpMessage;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)>{
    let secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            return Err((
                ErrorInternalServerError("JWT configuration is missing"),
                req,
            ));
        }
    };
    
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = match decode::<Claims>(credentials.token(), &DecodingKey::from_secret(secret.as_bytes()), &validation) {
        Ok(data) => data,
        Err(_) => {
            return Err((ErrorUnauthorized("Invalid or expired token"), req));
        }
    };
    
    req.extensions_mut().insert(token_data.claims);
    
    Ok(req)
}

pub async fn admin_validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)>{
    let secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            return Err((ErrorInternalServerError("JWT configuration is missing"), req));
        }
    };

    let validation = Validation::new(Algorithm::HS256);

    let token_data = match decode::<Claims>(credentials.token(), &DecodingKey::from_secret(secret.as_bytes()), &validation) {
        Ok(data) => data,
        Err(_) => {
            return Err((ErrorUnauthorized("Invalid or expired token"), req));
        }
    };

    if token_data.claims.role != "ADMIN"{
        return Err((ErrorForbidden("Admin access required"), req));
    }

    req.extensions_mut().insert(token_data.claims);

    Ok(req)
}