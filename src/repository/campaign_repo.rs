use async_trait::async_trait;
use sqlx::PgPool;
use chrono::Utc;
use crate::model::campaign::{Campaign, CampaignStatus, NewCampaignRequest, UpdateCampaignRequest};
use crate::errors::AppError;

#[async_trait]
pub trait CampaignRepository: Send + Sync {
    async fn create(&self, user_id: i32, campaign: &NewCampaignRequest) -> Result<Campaign, AppError>;
    async fn find_by_id(&self, campaign_id: i32) -> Result<Option<Campaign>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Campaign>, AppError>;
    async fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<Campaign>, AppError>;
    async fn update(&self, campaign_id: i32, user_id: i32, update: &UpdateCampaignRequest) -> Result<Campaign, AppError>;
    async fn delete(&self, campaign_id: i32, user_id: i32) -> Result<u64, AppError>;
    async fn update_status(&self, campaign_id: i32, status: CampaignStatus) -> Result<Campaign, AppError>;
    async fn upload_evidence(&self, campaign_id: i32, user_id: i32, evidence_url: &str) -> Result<Campaign, AppError>;
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
    async fn create(&self, user_id: i32, campaign: &NewCampaignRequest) -> Result<Campaign, AppError> {
        unimplemented!()
    }

    async fn find_by_id(&self, campaign_id: i32) -> Result<Option<Campaign>, AppError> {
        unimplemented!()
    }

    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
        unimplemented!()
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Campaign>, AppError> {
        unimplemented!()
    }

    async fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<Campaign>, AppError> {
        unimplemented!()
    }

    async fn update(&self, campaign_id: i32, user_id: i32, update: &UpdateCampaignRequest) -> Result<Campaign, AppError> {
        unimplemented!()
    }

    async fn delete(&self, campaign_id: i32, user_id: i32) -> Result<u64, AppError> {
        unimplemented!()
    }

    async fn update_status(&self, campaign_id: i32, status: CampaignStatus) -> Result<Campaign, AppError> {
        unimplemented!()
    }

    async fn upload_evidence(&self, campaign_id: i32, user_id: i32, evidence_url: &str) -> Result<Campaign, AppError> {
        unimplemented!()
    }
}