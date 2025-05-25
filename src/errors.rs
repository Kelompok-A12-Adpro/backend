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
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
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