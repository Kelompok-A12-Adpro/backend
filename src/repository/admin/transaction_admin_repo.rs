use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::admin::transaction_admin::{AdminTransactionView, TransactionFilterRequest};

#[async_trait]
pub trait TransactionAdminRepository: Send + Sync {
    async fn get_all_transactions(&self, filter: Option<TransactionFilterRequest>) -> Result<Vec<AdminTransactionView>, AppError>;
    async fn get_transaction_by_id(&self, transaction_id: i32) -> Result<Option<AdminTransactionView>, AppError>;
}

// Implementation will be added later
