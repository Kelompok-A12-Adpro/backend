use rocket::{response::Responder, http::{Status, ContentType}, Response, Request};
use thiserror::Error;
use serde_json::json;
use std::io::Cursor;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("JSON parsing error: {0}")]
    JsonParseError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String), // Keep original message for logging

    #[error("Database constraint violation: {0}")]
    DatabaseConstraintViolation(String), // Keep original message for logging

    #[error("Internal server error: {0}")]
    InternalServerError(String), // Keep original message for logging
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
        let (status, client_error_message) = match self {
            AppError::NotFound(msg) => (Status::NotFound, msg),
            AppError::ValidationError(msg) => (Status::BadRequest, msg),
            AppError::Forbidden(msg) => (Status::Forbidden, msg),
            AppError::Unauthorized => (Status::Unauthorized, "Authentication required.".to_string()),
            AppError::InvalidOperation(msg) => (Status::BadRequest, msg),
            AppError::JsonParseError(msg) => (Status::BadRequest, msg), // serde_json messages are often okay for clients

            AppError::DatabaseError(internal_msg) => {
                eprintln!("Database error detail: {}", internal_msg); // Log the specific error
                (Status::InternalServerError, "A database error occurred.".to_string()) // Generic message for client
            },
            AppError::DatabaseConstraintViolation(internal_msg) => {
                eprintln!("Database Constraint Violation detail: {}", internal_msg); // Log the specific error
                // You could try to make this message more specific if you parse internal_msg,
                // but a generic one is safer.
                (Status::Conflict, "The request conflicts with existing data.".to_string())
            },
            AppError::InternalServerError(internal_msg) => {
                eprintln!("Internal server error detail: {}", internal_msg); // Log the specific error
                (Status::InternalServerError, "An internal server error occurred.".to_string()) // Generic message for client
            },
        };

        // Create the JSON body
        let json_body = json!({ "error": client_error_message }).to_string();

        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(json_body.len(), Cursor::new(json_body))
            .ok()
    }
}