use rocket::{get, post, delete, routes, Route, State};
use rocket::serde::json::Json;

use crate::auth::AuthUser;
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet;
use crate::model::wallet::transaction::Transaction;
use crate::service::wallet::wallet_service::WalletService;

#[derive(serde::Deserialize)]
pub struct TopUpRequest {
    pub method: String,         // GOPAY or DANA
    pub phone_number: String,
    pub amount: f64,
}

#[derive(serde::Deserialize)]
pub struct WithdrawRequest {
    pub campaign_id: i32,
}

#[get("/wallet/me")]
pub async fn get_my_wallet(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
) -> Result<Json<Wallet>, AppError> {
    let wallet = wallet_service.get_wallet(auth_user.id).await?;
    Ok(Json(wallet))
}

#[post("/wallet/topup", format = "json", data = "<req>")]
pub async fn top_up_wallet(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
    req: Json<TopUpRequest>,
) -> Result<Json<Wallet>, AppError> {
    let wallet = wallet_service
        .top_up(
            auth_user.id, 
            &req.method, 
            &req.phone_number, 
            req.amount
        )
        .await?;
    Ok(Json(wallet))
}

#[get("/wallet/transactions")]
pub async fn get_transactions(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    let transactions = wallet_service.get_transactions(auth_user.id).await?;
    Ok(Json(transactions))
}

#[delete("/wallet/transactions/<transaction_id>")]
pub async fn delete_transaction(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
    transaction_id: i32,
) -> Result<Json<()>, AppError> {
    wallet_service.delete_transaction(auth_user.id, transaction_id).await?;
    Ok(Json(()))
}

#[post("/wallet/withdraw", format = "json", data = "<req>")]
pub async fn withdraw_funds(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
    req: Json<WithdrawRequest>,
) -> Result<Json<f64>, AppError> {
    let amount = wallet_service
        .withdraw_campaign_funds(auth_user.id, req.campaign_id)
        .await?;
    Ok(Json(amount))
}

pub fn wallet_routes() -> Vec<Route> {
    routes![
        get_my_wallet,
        top_up_wallet,
        get_transactions,
        delete_transaction,
        withdraw_funds,
    ]
}
