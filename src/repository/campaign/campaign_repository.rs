use async_trait::async_trait;
use sqlx::PgPool;
use chrono::{DateTime, Utc};
use crate::model::campaign::campaign::{Campaign, CampaignStatus};
use crate::errors::AppError;
use crate::repository::campaign::campaign_repository::CampaignRepository;

pub struct PgCampaignRepository {
    pool: PgPool,
}

impl PgCampaignRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl CampaignRepository for PgCampaignRepository {
    async fn create_campaign(&self, mut campaign: Campaign) -> Result<Campaign, AppError> {
        let rec = sqlx::query_as!(
            Campaign,
            r#"
            INSERT INTO campaigns 
              (user_id, name, description, target_amount, end_date, image_url, status, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING id, user_id, name, description, target_amount, collected_amount, end_date, image_url, status as "status: CampaignStatus", created_at, updated_at, evidence_url, evidence_uploaded_at
            "#,
            campaign.user_id,
            campaign.name,
            campaign.description,
            campaign.target_amount,
            campaign.end_date,
            campaign.image_url,
            campaign.status as CampaignStatus,
            campaign.created_at,
            campaign.updated_at
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError> {
        let rec = sqlx::query_as!(
            Campaign,
            "SELECT * FROM campaigns WHERE id=$1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn update_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError> {
        let rec = sqlx::query_as!(
            Campaign,
            r#"
            UPDATE campaigns
               SET name=$2, description=$3, target_amount=$4, end_date=$5, image_url=$6, status=$7, updated_at=$8
             WHERE id=$1
             RETURNING *  
            "#,
            campaign.id,
            campaign.name,
            campaign.description,
            campaign.target_amount,
            campaign.end_date,
            campaign.image_url,
            campaign.status as CampaignStatus,
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError> {
        let res = sqlx::query!("UPDATE campaigns SET status=$2, updated_at=$3 WHERE id=$1", id, status as _, Utc::now())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() == 1)
    }

    async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
        let recs = sqlx::query_as!(Campaign, "SELECT * FROM campaigns WHERE user_id=$1", user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(recs)
    }

    async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError> {
        let recs = sqlx::query_as!(Campaign, "SELECT * FROM campaigns WHERE status=$1", status as _)
            .fetch_all(&self.pool)
            .await?;
        Ok(recs)
    }

    async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError> {
        let recs = sqlx::query_as!(Campaign, "SELECT * FROM campaigns")
            .fetch_all(&self.pool)
            .await?;
        Ok(recs)
    }

    async fn delete_campaign(&self, id: i32) -> Result<bool, AppError> {
        let res = sqlx::query!("DELETE FROM campaigns WHERE id=$1", id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() == 1)
    }
}