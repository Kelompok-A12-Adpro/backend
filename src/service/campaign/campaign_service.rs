use std::sync::Arc;
use crate::model::campaign::campaign::{Campaign, CampaignStatus};
use crate::repository::campaign::campaign_repository::CampaignRepository;
use crate::service::campaign::factory::campaign_factory::CampaignFactory;
use crate::service::campaign::observer::campaign_observer::CampaignNotifier;
use crate::service::campaign::state::campaign_state::{CampaignState, PendingState, ActiveState, RejectedState, CompletedState};
use crate::errors::AppError;

pub struct CampaignService {
    repository: Arc<dyn CampaignRepository>,
    factory: Arc<CampaignFactory>,
    notifier: Arc<CampaignNotifier>,
}

impl CampaignService {
    pub fn new(
        repository: Arc<dyn CampaignRepository>,
        factory: Arc<CampaignFactory>,
        notifier: Arc<CampaignNotifier>,
    ) -> Self {
        CampaignService {
            repository,
            factory,
            notifier,
        }
    }
    
    pub async fn create_campaign(
        &self,
        user_id: i32,
        name: String,
        description: String,
        target_amount: f64,
    ) -> Result<Campaign, AppError> {
        let campaign = self.factory.create_campaign(user_id, name, description, target_amount);
        self.repository.create_campaign(campaign).await
    }
    
    pub async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError> {
        self.repository.get_campaign(id).await
    }
    
    pub async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
        self.repository.get_campaigns_by_user(user_id).await
    }
    
    pub async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError> {
        self.repository.get_campaigns_by_status(status).await
    }
    
    pub async fn approve_campaign(&self, id: i32) -> Result<Campaign, AppError> {
        let mut campaign = match self.repository.get_campaign(id).await? {
            Some(c) => c,
            None => return Err(AppError::NotFound(format!("Campaign with id {} not found", id))),
        };
        
        let old_status = campaign.status.clone();
        
        match campaign.status {
            CampaignStatus::PendingVerification => {
                let state = PendingState {};
                state.approve(&mut campaign)?;
            },
            CampaignStatus::Rejected => {
                let state = RejectedState {};
                state.approve(&mut campaign)?;
            },
            _ => return Err(AppError::InvalidOperation(format!("Cannot approve campaign in {:?} state", campaign.status))),
        }
        
        // Update campaign in repository
        let updated = self.repository.update_campaign(campaign.clone()).await?;
        
        // Notify observers
        self.notifier.notify_status_change(&updated, old_status);
        
        Ok(updated)
    }
    
    pub async fn reject_campaign(&self, id: i32, reason: Option<String>) -> Result<Campaign, AppError> {
        let mut campaign = match self.repository.get_campaign(id).await? {
            Some(c) => c,
            None => return Err(AppError::NotFound(format!("Campaign with id {} not found", id))),
        };
        
        let old_status = campaign.status.clone();
        
        match campaign.status {
            CampaignStatus::PendingVerification => {
                let state = PendingState {};
                state.reject(&mut campaign)?;
                // Store rejection reason if provided
            },
            _ => return Err(AppError::InvalidOperation(format!("Cannot reject campaign in {:?} state", campaign.status))),
        }
        
        // Update campaign in repository
        let updated = self.repository.update_campaign(campaign.clone()).await?;
        
        // Notify observers
        self.notifier.notify_status_change(&updated, old_status);
        
        Ok(updated)
    }
    
    // TODO: Add more methods for other state transitions...
}