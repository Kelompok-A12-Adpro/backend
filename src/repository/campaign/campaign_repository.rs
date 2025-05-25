use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use crate::model::campaign::campaign::{Campaign, CampaignStatus};
use crate::errors::AppError;

#[async_trait]
pub trait CampaignRepository: Send + Sync {
    async fn create_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>;
    async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError>;
    async fn update_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>;
    async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError>;
    async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError>;
    async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError>;
    async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError>;
    async fn delete_campaign(&self, id: i32) -> Result<bool, AppError>;
}

pub struct PgCampaignRepository {
    pool: PgPool,
}

impl PgCampaignRepository {
    pub fn new(pool: PgPool) -> Self {
        PgCampaignRepository { pool }
    }
}

#[async_trait]
impl CampaignRepository for PgCampaignRepository {
    async fn create_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError> {
        let rec = sqlx::query_as!(
            Campaign,
            "
            INSERT INTO campaigns
              (user_id,name,description,target_amount,collected_amount,start_date,end_date,image_url,status,created_at,updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING
              id,
              user_id,
              name,
              description,
              target_amount AS \"target_amount!\",
              collected_amount AS \"collected_amount!\",
              start_date AT TIME ZONE 'UTC' as \"start_date!\",
              end_date AT TIME ZONE 'UTC' as \"end_date!\",
              image_url,
              evidence_url,
              evidence_uploaded_at AT TIME ZONE 'UTC' as evidence_uploaded_at,
              status as \"status: CampaignStatus\",
              created_at AT TIME ZONE 'UTC' as \"created_at!\",
              updated_at AT TIME ZONE 'UTC' as \"updated_at!\"
            ",
            campaign.user_id,
            campaign.name,
            campaign.description,
            campaign.target_amount,
            campaign.collected_amount,
            campaign.start_date.naive_utc(),
            campaign.end_date.naive_utc(),
            campaign.image_url,
            campaign.status.to_string(),
            campaign.created_at.naive_utc(),
            campaign.updated_at.naive_utc(),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError> {
        let rec = sqlx::query_as!(
            Campaign,
            "
            SELECT
              id,
              user_id,
              name,
              description,
              target_amount AS \"target_amount!\",
              collected_amount AS \"collected_amount!\",
              start_date AT TIME ZONE 'UTC' as \"start_date!\",
              end_date AT TIME ZONE 'UTC' as \"end_date!\",
              image_url,
              evidence_url,
              evidence_uploaded_at AT TIME ZONE 'UTC' as evidence_uploaded_at,
              status as \"status: CampaignStatus\",
              created_at AT TIME ZONE 'UTC' as \"created_at!\",
              updated_at AT TIME ZONE 'UTC' as \"updated_at!\"
            FROM campaigns
            WHERE id = $1
            ",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn update_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError> {
        let rec = sqlx::query_as!(
            Campaign,
            "
            UPDATE campaigns
               SET name            = $2,
                   description     = $3,
                   target_amount   = $4,
                   collected_amount= $5,
                   start_date      = $6,
                   end_date        = $7,
                   image_url       = $8,
                   status          = $9,
                   updated_at      = $10
             WHERE id = $1
             RETURNING
               id,
               user_id,
               name,
               description,
               target_amount AS \"target_amount!\",
               collected_amount AS \"collected_amount!\",
               start_date AT TIME ZONE 'UTC' as \"start_date!\",
               end_date AT TIME ZONE 'UTC' as \"end_date!\",
               image_url,
               evidence_url,
               evidence_uploaded_at AT TIME ZONE 'UTC' as evidence_uploaded_at,
               status as \"status: CampaignStatus\",
               created_at AT TIME ZONE 'UTC' as \"created_at!\",
               updated_at AT TIME ZONE 'UTC' as \"updated_at!\"
            ",
            campaign.id,
            campaign.name,
            campaign.description,
            campaign.target_amount,
            campaign.collected_amount,
            campaign.start_date.naive_utc(),
            campaign.end_date.naive_utc(),
            campaign.image_url,
            campaign.status.to_string(),
            Utc::now().naive_utc(),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError> {
        let res = sqlx::query!(
            "UPDATE campaigns SET status = $1, updated_at = $2 WHERE id = $3",
            status.to_string(),
            Utc::now().naive_utc(),
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() == 1)
    }

    async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
        let recs = sqlx::query_as!(
            Campaign,
            "
            SELECT
              id,
              user_id,
              name,
              description,
              target_amount AS \"target_amount!\",
              collected_amount AS \"collected_amount!\",
              start_date AT TIME ZONE 'UTC' as \"start_date!\",
              end_date AT TIME ZONE 'UTC' as \"end_date!\",
              image_url,
              evidence_url,
              evidence_uploaded_at AT TIME ZONE 'UTC' as evidence_uploaded_at,
              status as \"status: CampaignStatus\",
              created_at AT TIME ZONE 'UTC' as \"created_at!\",
              updated_at AT TIME ZONE 'UTC' as \"updated_at!\"
            FROM campaigns
            WHERE user_id = $1
            ",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError> {
        let status_str = status.to_string();
        let recs = sqlx::query_as!(
            Campaign,
            "
            SELECT
              id,
              user_id,
              name,
              description,
              target_amount AS \"target_amount!\",
              collected_amount AS \"collected_amount!\",
              start_date AT TIME ZONE 'UTC' as \"start_date!\",
              end_date AT TIME ZONE 'UTC' as \"end_date!\",
              image_url,
              evidence_url,
              evidence_uploaded_at AT TIME ZONE 'UTC' as evidence_uploaded_at,
              status as \"status: CampaignStatus\",
              created_at AT TIME ZONE 'UTC' as \"created_at!\",
              updated_at AT TIME ZONE 'UTC' as \"updated_at!\"
            FROM campaigns
            WHERE status = $1
            ",
            status_str
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError> {
        let recs = sqlx::query_as!(
            Campaign,
            "
            SELECT
              id,
              user_id,
              name,
              description,
              target_amount AS \"target_amount!\",
              collected_amount AS \"collected_amount!\",
              start_date AT TIME ZONE 'UTC' as \"start_date!\",
              end_date AT TIME ZONE 'UTC' as \"end_date!\",
              image_url,
              evidence_url,
              evidence_uploaded_at AT TIME ZONE 'UTC' as evidence_uploaded_at,
              status as \"status: CampaignStatus\",
              created_at AT TIME ZONE 'UTC' as \"created_at!\",
              updated_at AT TIME ZONE 'UTC' as \"updated_at!\"
            FROM campaigns
            "
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    async fn delete_campaign(&self, id: i32) -> Result<bool, AppError> {
        let res = sqlx::query!("DELETE FROM campaigns WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() == 1)
    }
}