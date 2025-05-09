use rocket::{get, routes};
use rocket::serde::json::Json;
use crate::errors::AppError;
use std::collections::HashMap;

#[get("/transactions")]
fn get_transactions() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

#[get("/transactions/<_transaction_id>")]
fn get_transaction_details(_transaction_id: i32) -> Result<Json<HashMap<String, String>>, AppError> {
    Err(AppError::NotFound("Transaction not found".to_string()))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_transactions,
        get_transaction_details
    ]
}
