use rocket::{get, put, routes};
use rocket::serde::json::Json;
use crate::model::admin::fund_usage::{AdminFundUsageView, FundUsageVerificationRequest};
use crate::errors::AppError;

// Placeholder for simplified controllers
#[get("/fund-usage")]
fn get_fund_usages() -> Json<Vec<AdminFundUsageView>> {
    Json(Vec::new())
}

#[get("/fund-usage/<_usage_id>")]
fn get_fund_usage_details(_usage_id: i32) -> Result<Json<AdminFundUsageView>, AppError> {
    Err(AppError::NotFound("Fund usage record not found".to_string()))
}

#[put("/fund-usage/<_usage_id>/verify", format = "json", data = "<_verification_req>")]
fn verify_fund_usage(_usage_id: i32, _verification_req: Json<FundUsageVerificationRequest>) -> Result<Json<AdminFundUsageView>, AppError> {
    Err(AppError::NotFound("Fund usage record not found".to_string()))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_fund_usages,
        get_fund_usage_details,
        verify_fund_usage
    ]
}
