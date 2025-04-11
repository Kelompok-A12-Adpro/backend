use rocket::{response::Responder, http::Status, Response, Request};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error), // Example

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Authentication required")]
    Unauthorized,

}


#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        
        let status = match self {
            AppError::DatabaseError(_) => Status::InternalServerError,
            AppError::NotFound(_) => Status::NotFound,
            AppError::ValidationError(_) => Status::BadRequest,
            AppError::Forbidden(_) => Status::Forbidden,
            AppError::Unauthorized => Status::Unauthorized,
        };
        
        Response::build().status(status).ok()
    }
}