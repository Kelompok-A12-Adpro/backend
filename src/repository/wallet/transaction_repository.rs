use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::wallet::transaction::Transaction;

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create_transaction(&self, txn: Transaction) -> Result<Transaction, AppError>;
    async fn get_wallet_transactions(&self, wallet_id: i32) -> Result<Vec<Transaction>, AppError>;
    async fn delete_transaction(&self, transaction_id: i32, wallet_id: i32) -> Result<(), AppError>;
    async fn get_campaign_donations(&self, campaign_id: i32) -> Result<Vec<Transaction>, AppError>;}
