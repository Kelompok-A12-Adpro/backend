use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool, Transaction as SqlxTransaction, Postgres}; // Import SqlxError, Transaction
use crate::model::donation::donation::{Donation, NewDonationRequest};
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet; // For Wallet struct
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
    user_campaign_donation_cache: UserCampaignDonationCache,
}

impl PgDonationRepository {
    pub fn new(
        pool: PgPool,
        campaign_totals_cache: CampaignTotalsCache,
        user_campaign_donation_cache: UserCampaignDonationCache
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
        // Start a database transaction
        let mut tx: SqlxTransaction<Postgres> = self.pool.begin().await
            .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

        // --- 1. Wallet Operations ---
        // Lock the user's wallet row and fetch current details
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
        .fetch_one(&mut *tx) // Use the transaction object
        .await
        .map_err(|e| match e {
            SqlxError::RowNotFound => AppError::NotFound(format!("Wallet not found for user_id: {}", user_id)),
            _ => AppError::DatabaseError(format!("Failed to fetch or lock wallet for user_id {}: {}", user_id, e)),
        })?;

        // Ensure sufficient balance
        if wallet.balance < new_donation.amount as f64 {
            // No need to explicitly rollback, dropping tx will do it.
            return Err(AppError::DatabaseError("Insufficient wallet balance".to_string())); // Or a more specific AppError::InsufficientFunds
        }

        // Calculate new wallet balance
        let new_wallet_balance = wallet.balance - new_donation.amount as f64;

        // Update wallet balance and updated_at timestamp
        let wallet_update_result = sqlx::query!(
            r#"
            UPDATE wallets
            SET balance = $1, updated_at = NOW() AT TIME ZONE 'UTC'
            WHERE id = $2
            "#,
            new_wallet_balance,
            wallet.id
        )
        .execute(&mut *tx) // Use the transaction object
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update wallet balance for user_id {}: {}", user_id, e)))?;

        if wallet_update_result.rows_affected() == 0 {
            return Err(AppError::DatabaseError(format!("Failed to update wallet: no rows affected for wallet_id {}", wallet.id)));
        }

        // --- 2. Donation Creation ---
        // Insert the donation record
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
        .fetch_one(&mut *tx) // Use the transaction object
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create donation: {}", e)))?;

        // --- 3. Campaign Update ---
        // Update the campaign's collected_amount and updated_at timestamp
        let campaign_update_result = sqlx::query!(
            r#"
            UPDATE campaigns
            SET collected_amount = collected_amount + $1,
                updated_at = NOW() AT TIME ZONE 'UTC'
            WHERE id = $2
            "#,
            donation.amount, // Use amount from the created donation
            donation.campaign_id
        )
        .execute(&mut *tx) // Use the transaction object
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update campaign collected amount for campaign_id {}: {}", donation.campaign_id, e)))?;

        if campaign_update_result.rows_affected() == 0 {
             // This implies the campaign_id doesn't exist or there was an issue.
             // If a foreign key constraint exists from donations.campaign_id to campaigns.id,
             // the donation insert would have failed earlier if the campaign didn't exist.
             // So, this check is an additional safeguard or covers cases without FK.
            return Err(AppError::NotFound(format!("Campaign with id {} not found or failed to update collected amount.", donation.campaign_id)));
        }

        // --- 4. Transaction Logging ---
        // Log the donation to the transactions table
        // Assuming 'transactions' table's 'created_at' has a DEFAULT NOW() or similar
        // and 'is_deleted' should be false for new transactions.
        sqlx::query!(
            r#"
            INSERT INTO transactions (wallet_id, transaction_type, amount, campaign_id, is_deleted)
            VALUES ($1, 'donation', $2, $3, false)
            "#,
            wallet.id,
            donation.amount as f64, // Transaction amount is f64
            donation.campaign_id
        )
        .execute(&mut *tx) // Use the transaction object
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to log donation transaction: {}", e)))?;

        // --- Commit Transaction ---
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Transaction commit failed: {}", e)))?;

        // --- Cache Updates (After Successful Commit) ---
        // 1. Update CampaignTotalsCache
        {
            let mut campaign_totals_cache_guard = self.campaign_totals_cache.lock().await;
            let total_for_campaign = campaign_totals_cache_guard.entry(donation.campaign_id).or_insert(0);
            *total_for_campaign += donation.amount;
        }

        // 2. Update UserCampaignDonationCache
        {
            let mut user_campaign_donation_cache_guard = self.user_campaign_donation_cache.lock().await;
            let user_donations_to_campaigns = user_campaign_donation_cache_guard.entry(donation.user_id).or_default();
            let user_total_for_this_campaign = user_donations_to_campaigns.entry(donation.campaign_id).or_insert(0);
            *user_total_for_this_campaign += donation.amount;
        }

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
            SET message = $1, updated_at = NOW() AT TIME ZONE 'UTC' -- Assuming donations table has updated_at
            WHERE id = $2 AND user_id = $3
            "#,
            message,
            donation_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // If donation.updated_at is not a field, remove the SET updated_at part.
        // For this example, I'm assuming it's good practice to have it.
        // If your `Donation` model doesn't have `updated_at`, the SQL should be:
        // UPDATE donations SET message = $1 WHERE id = $2 AND user_id = $3

        Ok(result.rows_affected())
    }

    async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError> {
        {
            let cache_guard = self.campaign_totals_cache.lock().await;
            if let Some(total) = cache_guard.get(&campaign_id) {
                return Ok(*total);
            }
        }

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

        let total_amount = result.total_amount.unwrap_or(0);

        let mut cache_guard = self.campaign_totals_cache.lock().await;
        cache_guard.insert(campaign_id, total_amount);

        Ok(total_amount)
    }

    async fn get_user_total_for_campaign(&self, user_id: i32, campaign_id: i32) -> Result<i64, AppError> {
        {
            let cache_guard = self.user_campaign_donation_cache.lock().await;
            if let Some(user_donations) = cache_guard.get(&user_id) {
                if let Some(total) = user_donations.get(&campaign_id) {
                    return Ok(*total);
                }
            }
        }

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

        let mut cache_guard = self.user_campaign_donation_cache.lock().await;
        cache_guard.entry(user_id).or_default().insert(campaign_id, total_amount);

        Ok(total_amount)
    }
}