use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet;

#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: i32) -> Result<Option<Wallet>, AppError>;
    async fn update_balance(&self, user_id: i32, new_balance: f64) -> Result<(), AppError>;
    async fn create_wallet_if_not_exists(&self, user_id: i32) -> Result<Wallet, AppError>;}
