use rocket::{get, routes};
use rocket::serde::json::Json;
use crate::model::admin::transaction_admin::AdminTransactionView;
use crate::errors::AppError;

#[get("/transactions")]
fn get_transactions() -> Json<Vec<AdminTransactionView>> {
    Json(Vec::new())
}

#[get("/transactions/<_transaction_id>")]
fn get_transaction_details(_transaction_id: i32) -> Result<Json<AdminTransactionView>, AppError> {
    Err(AppError::NotFound("Transaction not found".to_string()))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_transactions,
        get_transaction_details
    ]
}
