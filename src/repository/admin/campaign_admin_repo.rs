use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::admin::campaign_admin::{AdminCampaignSummary, CampaignStatus};

#[async_trait]
pub trait CampaignAdminRepository: Send + Sync {
    async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<AdminCampaignSummary>, AppError>;
    async fn get_all_campaigns(&self) -> Result<Vec<AdminCampaignSummary>, AppError>;
    async fn get_campaign_details(&self, campaign_id: i32) -> Result<Option<AdminCampaignSummary>, AppError>;
    async fn update_campaign_status(&self, campaign_id: i32, status: CampaignStatus, reason: Option<String>) -> Result<bool, AppError>;
}

// Implementation will be added later
