use crate::errors::AppError;
use crate::model::campaign::{Campaign, CampaignStatus, NewCampaignRequest, UpdateCampaignRequest};
use crate::repository::campaign_repo::CampaignRepository;
use chrono::{Duration, Utc};
use std::sync::Arc;

pub struct CampaignService {
    campaign_repo: Arc<dyn CampaignRepository>,
}

impl CampaignService {
    pub fn new(campaign_repo: Arc<dyn CampaignRepository>) -> Self {
        CampaignService { campaign_repo }
    }

    pub async fn create_campaign(
        &self,
        user_id: i32,
        req: NewCampaignRequest,
    ) -> Result<Campaign, AppError> {
        // Basic validation
        if req.name.trim().is_empty() {
            return Err(AppError::ValidationError("Campaign name is required".to_string()));
        }

        if req.target_amount <= 0.0 {
            return Err(AppError::ValidationError(
                "Target amount must be positive".to_string(),
            ));
        }

        // Create campaign with pending verification status
        self.campaign_repo.create(user_id, &req).await
    }

    pub async fn get_campaign(&self, campaign_id: i32) -> Result<Campaign, AppError> {
        let campaign = self
            .campaign_repo
            .find_by_id(campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        Ok(campaign)
    }

    pub async fn get_user_campaigns(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
        self.campaign_repo.find_by_user(user_id).await
    }

    pub async fn update_campaign(
        &self,
        user_id: i32,
        campaign_id: i32,
        req: UpdateCampaignRequest,
    ) -> Result<Campaign, AppError> {
        // Verify campaign exists and belongs to the user
        let campaign = self
            .campaign_repo
            .find_by_id(campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        if campaign.user_id != user_id {
            return Err(AppError::Forbidden(
                "You don't have permission to update this campaign".to_string(),
            ));
        }

        // Perform update and reset to pending verification
        let updated_campaign = self.campaign_repo.update(campaign_id, user_id, &req).await?;
        self.campaign_repo
            .update_status(campaign_id, CampaignStatus::PendingVerification)
            .await
    }

    pub async fn delete_campaign(&self, user_id: i32, campaign_id: i32) -> Result<(), AppError> {
        // Verify campaign exists
        let campaign = self
            .campaign_repo
            .find_by_id(campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        // Check if user owns the campaign
        if campaign.user_id != user_id {
            return Err(AppError::Forbidden(
                "You don't have permission to delete this campaign".to_string(),
            ));
        }

        // Check deletion conditions:
        // 1. Status is PendingVerification OR
        // 2. Evidence was uploaded at least 30 days ago
        let can_delete = match campaign.status {
            CampaignStatus::PendingVerification => true,
            _ => {
                if let Some(evidence_date) = campaign.evidence_uploaded_at {
                    let days_since_evidence = (Utc::now() - evidence_date).num_days();
                    days_since_evidence >= 30
                } else {
                    false
                }
            }
        };

        if !can_delete {
            return Err(AppError::Forbidden(
                "This campaign cannot be deleted at this time".to_string(),
            ));
        }

        // Delete the campaign
        let rows_affected = self.campaign_repo.delete(campaign_id, user_id).await?;
        if rows_affected == 0 {
            return Err(AppError::NotFound("Campaign not found or already deleted".to_string()));
        }

        Ok(())
    }

    pub async fn upload_evidence(
        &self,
        user_id: i32,
        campaign_id: i32,
        evidence_url: String,
    ) -> Result<Campaign, AppError> {
        // Verify campaign exists and belongs to user
        let campaign = self
            .campaign_repo
            .find_by_id(campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        if campaign.user_id != user_id {
            return Err(AppError::Forbidden(
                "You don't have permission to upload evidence for this campaign".to_string(),
            ));
        }

        // Evidence must be a valid URL
        if evidence_url.trim().is_empty() {
            return Err(AppError::ValidationError("Evidence URL cannot be empty".to_string()));
        }

        // Upload the evidence
        self.campaign_repo
            .upload_evidence(campaign_id, user_id, &evidence_url)
            .await
    }
}