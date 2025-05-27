use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool}; // Import SqlxError
use crate::model::donation::donation::{Donation, NewDonationRequest}; // Assuming this path is correct
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet; // Assuming this path is correct

#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
}

pub struct PgDonationRepository {
    pool: PgPool,
}

impl PgDonationRepository {
    pub fn new(pool: PgPool) -> Self {
        PgDonationRepository { pool }
    }

    // Optional: Keep this if you need direct access elsewhere, but usually not necessary
    // pub fn pool(&self) -> &PgPool {
    //     &self.pool
    // }
}

#[async_trait]
impl DonationRepository for PgDonationRepository {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // 1. Lock wallet by user_id
        let wallet = sqlx::query_as!(
            Wallet,
            r#"
            SELECT id, user_id, balance, updated_at
            FROM wallets
            WHERE user_id = $1
            FOR UPDATE
            "#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Wallet not found: {}", e)))?;

        // 2. Ensure sufficient balance
        if wallet.balance < new_donation.amount as f64 {
            return Err(AppError::DatabaseError("Insufficient wallet balance".into()));
        }

        let new_balance = wallet.balance - new_donation.amount as f64;

        // 3. Update wallet balance
        sqlx::query!(
            r#"
            UPDATE wallets
            SET balance = $1
            WHERE id = $2
            "#,
            new_balance,
            wallet.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update wallet: {}", e)))?;

        // 4. Insert donation
        let donation = sqlx::query_as!(
            Donation,
            r#"
            INSERT INTO donations (user_id, campaign_id, amount, message)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, campaign_id, amount, message, created_at
            "#,
            user_id,
            new_donation.campaign_id,
            new_donation.amount,
            new_donation.message
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to insert donation: {}", e)))?;

        // 5. Log to transactions
        sqlx::query!(
            r#"
            INSERT INTO transactions (wallet_id, transaction_type, amount, campaign_id)
            VALUES ($1, 'donation', $2, $3)
            "#,
            wallet.id,
            new_donation.amount as f64,
            new_donation.campaign_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to insert transaction: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Commit failed: {}", e)))?;

        Ok(donation)
    }



    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError> {
        let donation = sqlx::query_as!(
            Donation,
            r#"
            SELECT id, user_id, campaign_id, amount, message, created_at
            FROM donations
            WHERE id = $1
            "#,
            donation_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(donation)
    }

    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError> {
        let donations = sqlx::query_as!(
            Donation,
            r#"
            SELECT id, user_id, campaign_id, amount, message, created_at
            FROM donations
            WHERE campaign_id = $1
            ORDER BY created_at DESC
            "#,
            campaign_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(donations)
    }

    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError> {
        let donations = sqlx::query_as!(
            Donation,
            r#"
            SELECT id, user_id, campaign_id, amount, message, created_at
            FROM donations
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(donations)
    }

    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError> {
        // We also check user_id to ensure a user can only update their own donation's message.
        let result = sqlx::query!(
            r#"
            UPDATE donations
            SET message = $1
            WHERE id = $2 AND user_id = $3
            "#,
            message,
            donation_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}