use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool}; // Import SqlxError
use crate::model::donation::donation::{Donation, NewDonationRequest}; // Assuming this path is correct
use crate::errors::AppError; // Assuming this path is correct

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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            SqlxError::Database(db_err) if db_err.constraint().is_some() => {
                AppError::DatabaseConstraintViolation(db_err.message().to_string()) // More specific error
            }
            _ => AppError::DatabaseError(e.to_string()),
        })?;
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