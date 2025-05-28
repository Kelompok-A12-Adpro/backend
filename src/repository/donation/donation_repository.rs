use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool}; // Import SqlxError
use crate::model::donation::donation::{Donation, NewDonationRequest}; // Assuming this path is correct
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet; // Assuming this path is correct
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type CampaignTotalsCache = Arc<Mutex<HashMap<i32, i64>>>;

// ... other imports ...
// use crate::model::donation::donation::{Donation, NewDonationRequest}; // Assuming this path is correct
// use crate::errors::AppError;
// use crate::model::wallet::wallet::Wallet; // Assuming this path is correct
// use std::collections::HashMap; // Already imported if using type alias
// use std::sync::Arc;
// use tokio::sync::Mutex;

// Your type alias from above
pub type CampaignTotalsCache = Arc<Mutex<HashMap<i32, i64>>>;


#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
    // New method to get the cached total
    async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError>;
}

pub struct PgDonationRepository {
    pool: PgPool,
    campaign_totals_cache: CampaignTotalsCache, // Add the cache field
}

impl PgDonationRepository {
    // Modify the constructor to accept the cache
    pub fn new(pool: PgPool, campaign_totals_cache: CampaignTotalsCache) -> Self {
        PgDonationRepository { pool, campaign_totals_cache }
    }
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
        .map_err(|e| {
            // It's good practice to roll back on error before returning
            // tx.rollback().await.ok(); // Or handle rollback error
            AppError::DatabaseError(format!("Wallet not found or lock failed: {}", e))
        })?;

        // 2. Ensure sufficient balance
        if wallet.balance < new_donation.amount as f64 {
            // tx.rollback().await.ok();
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
        .map_err(|e| {
            // tx.rollback().await.ok();
            AppError::DatabaseError(format!("Failed to update wallet: {}", e))
        })?;

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
        .map_err(|e| {
            // tx.rollback().await.ok();
            AppError::DatabaseError(format!("Failed to insert donation: {}", e))
        })?;

        // 5. Log to transactions
        sqlx::query!(
            r#"
            INSERT INTO transactions (wallet_id, transaction_type, amount, campaign_id)
            VALUES ($1, 'donation', $2, $3)
            "#,
            wallet.id,
            new_donation.amount as f64, // Ensure this matches DB type, might be new_donation.amount if DB is integer
            new_donation.campaign_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            // tx.rollback().await.ok();
            AppError::DatabaseError(format!("Failed to insert transaction: {}", e))
        })?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Commit failed: {}", e)))?;

        // --- START CACHE UPDATE ---
        // If commit was successful, update the cache
        // This happens outside the DB transaction. If this part fails, the DB is still consistent.
        // The cache might be slightly stale until the next read for this campaign_id populates it.
        let mut cache_guard = self.campaign_totals_cache.lock().await;
        let total_for_campaign = cache_guard.entry(donation.campaign_id).or_insert(0);
        *total_for_campaign += donation.amount;
        // --- END CACHE UPDATE ---

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

    // Implementation for the new trait method
    async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError> {
        // Try to get from cache first
        { // Scoped lock
            let cache_guard = self.campaign_totals_cache.lock().await;
            if let Some(total) = cache_guard.get(&campaign_id) {
                return Ok(*total);
            }
        } // Lock released here

        // If not in cache, compute from DB, then populate cache
        // This uses the existing find_by_campaign method.
        // Note: This could be slow if there are many donations per campaign.
        // For a production system, you might sum in the DB directly:
        // SELECT COALESCE(SUM(amount), 0) FROM donations WHERE campaign_id = $1
        let donations = self.find_by_campaign(campaign_id).await?;
        let total_amount: i64 = donations.iter().map(|d| d.amount).sum();

        // Populate the cache
        let mut cache_guard = self.campaign_totals_cache.lock().await;
        cache_guard.insert(campaign_id, total_amount);

        Ok(total_amount)
    }
}