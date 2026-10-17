use actix_web::{post, web, HttpResponse};
use serde::Serialize;
use validator::Validate;
use crate::models::auth::{LoginRequest,RegisterRequest};
use crate::services::auth::{self, LoginError, RegisterError};
use crate::state::AppState;
use crate::models::claims::Claims;
use actix_web::{HttpRequest, HttpMessage};


#[derive(Serialize)]
struct RegisterResponse{
    id: uuid::Uuid,
    email: String,
    role: String
}

#[post("/register")]
pub async fn register(state: web::Data<AppState>, body: web::Json<RegisterRequest>) -> HttpResponse{

    if let Err(errors) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "VALIDATION_ERROR",
            "message": "Please check your input",
            "details":errors.to_string()
        }));
    }

    match auth::register_user(&state.db, &body.email, &body.password).await {
        Ok(user) => {
            let response = RegisterResponse{
                id: user.id,
                email: user.email,
                role: user.role
            };

            HttpResponse::Created().json(response)
        }

        Err(RegisterError::EmailAlreadyExists) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error":"EMAIL_ALREADY_EXISTS",
                "message": "An account with this email already exists"
            }))
        }
        Err(RegisterError::PasswordHashing) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "PASSWORD_ALREADY_EXISTS",
                "message": "Could not process the registration"
            }))
        }
        Err(RegisterError::Database(_)) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "INTERNAL_SERVER_ERROR",
                "message": "Could not complete the registration"
            }))
        }
    }
}

#[derive(Serialize)]
struct LoginUserResponse{
    id: uuid::Uuid,
    email: String,
    role: String,
}

#[derive(Serialize)]
struct LoginResponse{
    access_token: String,
    token_type: &'static str,
    expires_in: u64,
    user: LoginUserResponse
}

#[post("/login")]
pub async fn login(state: web::Data<AppState>, body: web::Json<LoginRequest>)-> HttpResponse{
    if let Err(errors) = body.validate(){
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "VALIDATION_ERROR",
            "message": "Please check your input",
            "details": errors.to_string()
        }));
    }

    match auth::login_user(&state.db, &body.email, &body.password).await{
        Ok((user, token)) => {
            HttpResponse::Ok().json(LoginResponse{
                access_token: token,
                token_type: "Bearer",
                expires_in: 3600,
                user: LoginUserResponse{
                    id: user.id,
                    email: user.email,
                    role: user.role
                },
            })
        }

        Err(LoginError::InvalidCredentials)=>{
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "INVALID_CREDENTIALS",
                "message": "Email or password is incorrect"
            }))
        }

        Err(LoginError::Configuration | LoginError::TokenCreation) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "AUTHENTICATION_ERROR",
                "message": "Could not complete the login"
            }))
        }
    }
}


#[derive(Serialize)]
struct AccountResponse {
    id: String,
    role: String,
}

pub async fn me(req: HttpRequest) -> HttpResponse {
    let claims = {
        let extensions = req.extensions();
        extensions.get::<Claims>().cloned()
    };

    match claims {
        Some(claims) => {
            HttpResponse::Ok().json(AccountResponse {
                id: claims.sub,
                role: claims.role,
            })
        }
        None => {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "UNAUTHORIZED",
                "message": "Authentication required"
            }))
        }
    }
}



