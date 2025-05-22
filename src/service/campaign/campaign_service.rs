use std::sync::Arc;
use crate::model::campaign::campaign::{Campaign, CampaignStatus};
use crate::repository::campaign::campaign_repository::CampaignRepository;
use crate::service::campaign::factory::campaign_factory::CampaignFactory;
use crate::service::campaign::state::campaign_state::{CampaignState, PendingState, ActiveState, RejectedState, CompletedState};
use crate::errors::AppError;

pub struct CampaignService {
    repository: Arc<dyn CampaignRepository>,
    factory: Arc<CampaignFactory>,
}

impl CampaignService {
    pub fn new(
        repository: Arc<dyn CampaignRepository>,
        factory: Arc<CampaignFactory>,

    ) -> Self {
        CampaignService {
            repository,
            factory,
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
        match self.repository.get_campaign(id).await {
            Ok(opt)       => Ok(opt),
            Err(AppError::NotFound(_)) => Ok(None),
            Err(e)         => Err(e),
        }
    }
    
    pub async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
        self.repository.get_campaigns_by_user(user_id).await
    }
    
    pub async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError> {
        self.repository.get_campaigns_by_status(status).await
    }

    pub async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError> {
        self.repository.get_all_campaigns().await
    }
    
    pub async fn delete_campaign(&self, id: i32) -> Result<bool, AppError> {
        // Get the campaign to check its status before deletion
        let campaign = match self.repository.get_campaign(id).await? {
            Some(c) => c,
            None => return Err(AppError::NotFound(format!("Campaign with id {} not found", id))),
        };
        
        // Only allow deletion if campaign is in Pending or Rejected status
        match campaign.status {
            CampaignStatus::PendingVerification | CampaignStatus::Rejected => {
                // Proceed with deletion
                self.repository.delete_campaign(id).await
            },
            _ => Err(AppError::InvalidOperation(format!("Cannot delete campaign in {:?} state", campaign.status))),
        }
    }

    fn state_from_status(status: CampaignStatus) -> Box<dyn CampaignState + Send> {
        match status {
            CampaignStatus::PendingVerification => Box::new(PendingState),
            CampaignStatus::Active              => Box::new(ActiveState),
            CampaignStatus::Rejected            => Box::new(RejectedState),
            CampaignStatus::Completed           => Box::new(CompletedState),
        }
    }

    pub async fn fetch_or_404(&self, id: i32) -> Result<Campaign, AppError> {
        match self.get_campaign(id).await? {
            Some(campaign) => Ok(campaign),
            None => Err(AppError::NotFound(format!("Campaign with id {} not found", id))),
        }
    }

     pub async fn approve_campaign(&self, id: i32) -> Result<Campaign, AppError> {
        let mut campaign = self.fetch_or_404(id).await?;
        let old_status = campaign.status.clone();

        // sebelumnya: panggil state berdasarkan status lama
        let mut state = Self::state_from_status(old_status.clone());
        state = state.approve(&mut campaign)?;
        let mut state = Self::state_from_status(campaign.status.clone());
        state = state.approve(&mut campaign)?;

        let updated = self.repository.update_campaign(campaign).await?;
        Ok(updated)
    }

    pub async fn reject_campaign(&self, id: i32, reason: Option<String>) -> Result<Campaign, AppError> {
        let mut campaign = self.fetch_or_404(id).await?;
        let old_status = campaign.status.clone();

        let mut state = Self::state_from_status(old_status.clone());
        state = state.reject(&mut campaign)?;
        let mut state = Self::state_from_status(campaign.status.clone());
        state = state.reject(&mut campaign)?;

        let updated = self.repository.update_campaign(campaign).await?;
        Ok(updated)
    }

    pub async fn complete_campaign(&self, id: i32) -> Result<Campaign, AppError> {
        let mut campaign = self.fetch_or_404(id).await?;
        let old_status = campaign.status.clone();

        let mut state = Self::state_from_status(old_status.clone());
        state = state.complete(&mut campaign)?;
        let mut state = Self::state_from_status(campaign.status.clone());
        state = state.complete(&mut campaign)?;

        let updated = self.repository.update_campaign(campaign).await?;
        Ok(updated)
    }
    
    // TODO: Add more methods for other state transitions...
}