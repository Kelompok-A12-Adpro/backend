use async_trait::async_trait;
use sqlx::{PgPool, Error as SqlxError};
use crate::errors::AppError;
use crate::model::wallet::transaction::Transaction;

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create_transaction(&self, txn: Transaction) -> Result<Transaction, AppError>;
    async fn get_wallet_transactions(&self, wallet_id: i32) -> Result<Vec<Transaction>, AppError>;
    async fn delete_transaction(&self, transaction_id: i32, wallet_id: i32) -> Result<(), AppError>;
    async fn get_campaign_donations(&self, campaign_id: i32) -> Result<Vec<Transaction>, AppError>;
}

pub struct PgTransactionRepository {
    pool: PgPool,
}

impl PgTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        PgTransactionRepository { pool }
    }
}

#[async_trait]
impl TransactionRepository for PgTransactionRepository {
    async fn create_transaction(&self, txn: Transaction) -> Result<Transaction, AppError> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            INSERT INTO transactions (wallet_id, transaction_type, amount, method, phone_number, campaign_id, is_deleted)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, wallet_id, transaction_type, amount, method, phone_number, campaign_id, created_at, is_deleted
            "#,
            txn.wallet_id,
            txn.transaction_type,
            txn.amount,
            txn.method,
            txn.phone_number,
            txn.campaign_id,
            txn.is_deleted,
                )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(transaction)
    }

    async fn get_wallet_transactions(&self, wallet_id: i32) -> Result<Vec<Transaction>, AppError> {
        let txns = sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, 
                wallet_id, 
                transaction_type,
                amount, 
                method, 
                phone_number,
                campaign_id, 
                created_at,
                is_deleted
            FROM transactions
            WHERE wallet_id = $1
            ORDER BY created_at DESC
            "#,
            wallet_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(txns)
    }

    async fn delete_transaction(&self, transaction_id: i32, wallet_id: i32) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM transactions
            WHERE id = $1 AND wallet_id = $2
            "#,
            transaction_id,
            wallet_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_campaign_donations(&self, campaign_id: i32) -> Result<Vec<Transaction>, AppError> {
        let txns = sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, 
                wallet_id, 
                transaction_type,
                amount, 
                method, 
                phone_number,
                campaign_id, 
                created_at,
                is_deleted
            FROM transactions
            WHERE wallet_id = $1
            ORDER BY created_at DESC
            "#,
            campaign_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(txns)
    }
}
