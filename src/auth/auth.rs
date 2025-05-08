use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use crate::errors::AppError; // Assuming your AppError is here

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i32,
    // You could add other fields if your downstream code expects them,
    // e.g., username: String, roles: Vec<String>,
    // but for a simple placeholder, id is often sufficient.
}

/// Placeholder implementation for AuthUser.
///
/// In a real application, this would involve token validation, database lookups, etc.
/// For testing and development without a full auth system, this guard
/// will look for an "X-Test-User-Id" header.
/// If the header is present and a valid i32, it will create an AuthUser.
/// If the header is missing or invalid, it will return an Unauthorized error.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = AppError; // Use your application's error type

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Check for a specific header, e.g., "X-Test-User-Id"
        // This allows tests to specify which user is "logged in".
        if let Some(id_str) = req.headers().get_one("X-Test-User-Id") {
            match id_str.parse::<i32>() {
                Ok(id) => {
                    // Successfully parsed the user ID from the header
                    Outcome::Success(AuthUser { id })
                }
                Err(_) => {
                    // Header was present but not a valid i32
                    Outcome::Failure((
                        Status::BadRequest, // Or Unauthorized, depending on desired behavior
                        AppError::ValidationError("Invalid X-Test-User-Id header format.".to_string()),
                    ))
                }
            }
        } else {
            // Header is missing. For a placeholder, you might:
            // 1. Fail (most similar to real auth if no token is provided)
            Outcome::Failure((
                Status::Unauthorized,
                AppError::Unauthorized("Missing X-Test-User-Id header for placeholder authentication.".to_string()),
            ))
            // 2. Succeed with a default user ID (can simplify some tests, but less realistic)
            // Outcome::Success(AuthUser { id: 999 }) // Example default user
        }
    }
}