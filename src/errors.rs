use rocket::{response::Responder, http::{Status, ContentType}, Response, Request}; // Added ContentType
use thiserror::Error;
use serde_json::json; // Make sure serde_json is a dependency and imported
use std::io::Cursor;   // For the body

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Authentication required")]
    Unauthorized, // This one doesn't have a message field in your enum definition

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("JSON parsing error: {0}")]
    JsonParseError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Database constraint violation: {0}")]
    DatabaseConstraintViolation(String),
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (Status::NotFound, msg),
            AppError::ValidationError(msg) => (Status::BadRequest, msg),
            AppError::Forbidden(msg) => (Status::Forbidden, msg),
            AppError::Unauthorized => (Status::Unauthorized, "Authentication required".to_string()), // Provide a default message string
            AppError::InvalidOperation(msg) => (Status::BadRequest, msg),
            AppError::JsonParseError(msg) => (Status::BadRequest, msg),
            AppError::DatabaseError(msg) => (Status::InternalServerError, msg),
             AppError::DatabaseConstraintViolation(msg) => {
                // Log the specific constraint violation server-side for debugging if needed
                // eprintln!("Database Constraint Violation: {}", msg);
                (Status::Conflict, msg) // HTTP 409 Conflict is good for this
            }
        };

        // Create the JSON body
        let json_body = json!({ "error": error_message }).to_string();

        Response::build()
            .status(status)
            .header(ContentType::JSON) // Set the Content-Type header
            .sized_body(json_body.len(), Cursor::new(json_body)) // Set the JSON body
            .ok()
    }
}