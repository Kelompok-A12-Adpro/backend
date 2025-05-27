use async_trait::async_trait;
use sqlx::{PgPool, Error as SqlxError};
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet;

#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: i32) -> Result<Option<Wallet>, AppError>;
    async fn update_balance(&self, user_id: i32, new_balance: f64) -> Result<(), AppError>;
    async fn create_wallet_if_not_exists(&self, user_id: i32) -> Result<Wallet, AppError>;
}

pub struct PgWalletRepository {
    pool: PgPool,
}

impl PgWalletRepository {
    pub fn new(pool: PgPool) -> Self {
        PgWalletRepository { pool }
    }
}

#[async_trait]
impl WalletRepository for PgWalletRepository {
    async fn find_by_user_id(&self, user_id: i32) -> Result<Option<Wallet>, AppError> {
        let wallet = sqlx::query_as!(
                    Wallet,
                    r#"
                    SELECT id, user_id, balance, updated_at
                    FROM wallets
                    WHERE user_id = $1
                    "#,
                    user_id
                )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(wallet)
    }

    async fn update_balance(&self, user_id: i32, new_balance: f64) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE wallets
            SET balance = $1
            WHERE user_id = $2
            "#,
            new_balance,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_wallet_if_not_exists(&self, user_id: i32) -> Result<Wallet, AppError> {
        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            INSERT INTO wallets (user_id, balance)
            VALUES ($1, 0.0)
            ON CONFLICT (user_id) DO NOTHING
            RETURNING id, user_id, balance, updated_at
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some(w) = wallet {
            Ok(w)
        } else {
            // If already exists, fetch existing wallet
            self.find_by_user_id(user_id).await?
                .ok_or_else(|| AppError::NotFound(format!("Wallet for user {} not found", user_id)))
        }
    }
}
