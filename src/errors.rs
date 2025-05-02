use rocket::{response::Responder, http::Status, Response, Request};
use thiserror::Error;

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

    #[error("Internal error: {0}")]
    InternalServerError(String),
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        
        let status = match self {
            AppError::NotFound(_) => Status::NotFound,
            AppError::ValidationError(_) => Status::BadRequest,
            AppError::Forbidden(_) => Status::Forbidden,
            AppError::Unauthorized => Status::Unauthorized,
            AppError::InternalServerError(_) => Status::InternalServerError,
        };
        
        Response::build().status(status).ok()
    }
}