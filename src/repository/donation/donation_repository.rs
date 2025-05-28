use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool}; // Import SqlxError
use crate::model::donation::donation::{Donation, NewDonationRequest}; // Assuming this path is correct
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet; // Assuming this path is correct
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type CampaignTotalsCache = Arc<Mutex<HashMap<i32, i64>>>;
pub type UserCampaignDonationCache = Arc<Mutex<HashMap<i32, HashMap<i32, i64>>>>;

#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
    async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError>;
    async fn get_user_total_for_campaign(&self, user_id: i32, campaign_id: i32) -> Result<i64, AppError>;
}

pub struct PgDonationRepository {
    pool: PgPool,
    campaign_totals_cache: CampaignTotalsCache,
    user_campaign_donation_cache: UserCampaignDonationCache, // Add the new cache field
}

impl PgDonationRepository {
    pub fn new(
        pool: PgPool, 
        campaign_totals_cache: CampaignTotalsCache,
        user_campaign_donation_cache: UserCampaignDonationCache // New parameter
    ) -> Self {
        PgDonationRepository { 
            pool, 
            campaign_totals_cache,
            user_campaign_donation_cache 
        }
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

        // --- START CACHE UPDATES (after successful commit) ---
        // 1. Update CampaignTotalsCache (global total for the campaign)
        { // Scoped lock
            let mut global_cache_guard = self.campaign_totals_cache.lock().await;
            let total_for_campaign = global_cache_guard.entry(donation.campaign_id).or_insert(0);
            *total_for_campaign += donation.amount;
        }

        // 2. Update UserCampaignDonationCache (total for this user for this campaign)
        { // Scoped lock
            let mut user_cache_guard = self.user_campaign_donation_cache.lock().await;
            let user_donations_to_campaigns = user_cache_guard.entry(donation.user_id).or_default(); // Get or insert HashMap for user
            let user_total_for_this_campaign = user_donations_to_campaigns.entry(donation.campaign_id).or_insert(0);
            *user_total_for_this_campaign += donation.amount;
        }
        // --- END CACHE UPDATES ---

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

    // Implementation for CampaignTotalsCache accessor
    async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError> {
        // Try to get from cache first
        { // Scoped lock
            let cache_guard = self.campaign_totals_cache.lock().await;
            if let Some(total) = cache_guard.get(&campaign_id) {
                return Ok(*total);
            }
        }

        // If not in cache, compute from DB, then populate cache
        // This query directly sums in the DB for efficiency.
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total_amount
            FROM donations
            WHERE campaign_id = $1
            "#,
            campaign_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch total for campaign {}: {}", campaign_id, e)))?;
        
        let total_amount = result.total_amount.unwrap_or(0); // COALESCE handles NULL, sum returns Option<Decimal> or Option<i64>

        // Populate the cache
        let mut cache_guard = self.campaign_totals_cache.lock().await;
        cache_guard.insert(campaign_id, total_amount);

        Ok(total_amount)
    }

    // Implementation for UserCampaignDonationCache accessor
    async fn get_user_total_for_campaign(&self, user_id: i32, campaign_id: i32) -> Result<i64, AppError> {
        // Try to get from cache first
        { // Scoped lock
            let cache_guard = self.user_campaign_donation_cache.lock().await;
            if let Some(user_donations) = cache_guard.get(&user_id) {
                if let Some(total) = user_donations.get(&campaign_id) {
                    return Ok(*total);
                }
            }
        }

        // If not in cache, compute from DB, then populate cache
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total_amount
            FROM donations
            WHERE user_id = $1 AND campaign_id = $2
            "#,
            user_id,
            campaign_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user total for campaign {}: {}", campaign_id, e)))?;

        let total_amount = result.total_amount.unwrap_or(0);

        // Populate the cache
        let mut cache_guard = self.user_campaign_donation_cache.lock().await;
        cache_guard.entry(user_id).or_default().insert(campaign_id, total_amount);
        
        Ok(total_amount)
    }
}