use actix_web::{post, web, HttpResponse};
use serde::Serialize;
use validator::Validate;
use crate::models::auth::RegisterRequest;
use crate::services::auth::{self, RegisterError};
use crate::state::AppState;


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