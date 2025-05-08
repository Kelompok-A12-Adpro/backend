// src/auth/auth.rs (or src/auth/mod.rs)

// Ensure these use statements are correct and at the top of THIS file
use rocket::http::Status;
use rocket::request::{FromRequest, Request}; // Note: Outcome is NOT explicitly used here now
use crate::errors::AppError; // Make sure this path to AppError is correct

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = AppError;

    // Fully qualify Outcome in the return type
    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        const AUTH_HEADER_NAME: &str = "X-Test-User-Id";

        if let Some(id_str) = req.headers().get_one(AUTH_HEADER_NAME) {
            match id_str.parse::<i32>() {
                Ok(id) => {
                    // Fully qualify Outcome here
                    rocket::request::Outcome::Success(AuthUser { id })
                }
                Err(_) => {
                    // Fully qualify Outcome here
                    rocket::request::Outcome::Failure(
                        Status::BadRequest,
                        AppError::ValidationError(format!("Invalid {} header format.", AUTH_HEADER_NAME)),
                    )
                }
            }
        } else {
            // Fully qualify Outcome here
            rocket::request::Outcome::Failure(
                Status::Unauthorized,
                AppError::Unauthorized(format!("Missing {} header for placeholder authentication.", AUTH_HEADER_NAME)),
            )
        }
    }
}