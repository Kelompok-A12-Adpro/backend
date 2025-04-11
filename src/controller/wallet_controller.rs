use rocket::{get, post, delete, routes, Route, State};
use rocket::serde::json::Json;

use crate::auth::AuthUser;
use crate::errors::AppError;
use crate::model::wallet::Wallet;
use crate::model::wallet_transaction::WalletTransaction;
use crate::service::wallet_service::WalletService;
use crate::service::commands::wallet_commands::{
    TopUpWalletCommand, DeleteTopUpTransactionCommand,
};

#[derive(serde::Deserialize)]
pub struct TopUpRequest {
    pub method: String,
    pub phone_number: String,
    pub amount: f64,
}

#[post("/wallet/topup", format = "json", data = "<req>")]
pub async fn top_up_wallet_route(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
    req: Json<TopUpRequest>,
) -> Result<Json<WalletTransaction>, AppError> {
    let cmd = TopUpWalletCommand {
        user_id: auth_user.id,
        method: req.method.clone(),
        phone_number: req.phone_number.clone(),
        amount: req.amount,
    };

    let result = wallet_service.top_up(cmd).await?;
    Ok(Json(result))
}

#[get("/wallet/me")]
pub async fn get_my_wallet_route(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
) -> Result<Json<Wallet>, AppError> {
    let wallet = wallet_service.get_wallet(auth_user.id).await?;
    Ok(Json(wallet))
}

#[get("/wallet/transactions")]
pub async fn get_my_wallet_transactions_route(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
) -> Result<Json<Vec<WalletTransaction>>, AppError> {
    let txs = wallet_service.get_transaction_history(auth_user.id).await?;
    Ok(Json(txs))
}

#[delete("/wallet/transactions/<tx_id>")]
pub async fn delete_wallet_transaction_route(
    auth_user: AuthUser,
    wallet_service: &State<WalletService>,
    tx_id: i32,
) -> Result<(), AppError> {
    let cmd = DeleteTopUpTransactionCommand {
        user_id: auth_user.id,
        transaction_id: tx_id,
    };
    wallet_service.delete_topup_transaction(cmd).await?;
    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![
        top_up_wallet_route,
        get_my_wallet_route,
        get_my_wallet_transactions_route,
        delete_wallet_transaction_route
    ]
}
