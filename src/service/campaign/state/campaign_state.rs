use crate::model::campaign::campaign::{Campaign, CampaignStatus};
use crate::errors::AppError;

pub trait CampaignState {
    fn approve(&self, campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError>;
    fn reject(&self, campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError>;
    fn complete(&self, campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError>;
}

pub struct PendingState;
pub struct ActiveState;
pub struct RejectedState;
pub struct CompletedState;

impl CampaignState for PendingState {
    fn approve(&self, campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        campaign.status = CampaignStatus::Active;
        Ok(Box::new(ActiveState))
    }

    fn reject(&self, campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        campaign.status = CampaignStatus::Rejected;
        Ok(Box::new(RejectedState))
    }

    fn complete(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Cannot complete a pending campaign".to_string()))
    }

}

impl CampaignState for ActiveState {
    fn approve(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Campaign is already active".to_string()))
    }

    fn reject(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Cannot reject an active campaign".to_string()))
    }

    fn complete(&self, campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        campaign.status = CampaignStatus::Completed;
        Ok(Box::new(CompletedState))
    }
}

impl CampaignState for RejectedState {
    fn approve(&self, campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        campaign.status = CampaignStatus::Active;
        Ok(Box::new(ActiveState))
    }

    fn reject(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Campaign is already rejected".to_string()))
    }

    fn complete(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Cannot complete a rejected campaign".to_string()))
    }
}

impl CampaignState for CompletedState {

    fn approve(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Cannot approve a completed campaign".to_string()))
    }

    fn reject(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Cannot reject a completed campaign".to_string()))
    }

    fn complete(&self, _campaign: &mut Campaign) -> Result<Box<dyn CampaignState>, AppError> {
        Err(AppError::InvalidOperation("Campaign is already completed".to_string()))
    }
}
