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
        target_amount: i64,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        image_url: Option<String>,
    ) -> Result<Campaign, AppError> {
        let campaign = self.factory.create_campaign(user_id, name, description, target_amount, start_date, end_date, image_url);
        self.repository.create_campaign(campaign).await
    }

    pub async fn update_campaign(
        &self,
        id: i32,
        user_id: i32,
        name: Option<String>,
        description: Option<String>,
        target_amount: Option<i64>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        image_url: Option<String>,
    ) -> Result<Campaign, AppError> {
        // Fetch existing campaign
        let mut campaign = self.fetch_or_404(id).await?;
        
        // Check if user owns the campaign
        if campaign.user_id != user_id { // TODO: Change variabel user_id to check against the user_id of the authenticated user
            return Err(AppError::InvalidOperation("User not authorized to update this campaign".to_string()));
        }
        
        // Only allow updates if campaign is in PendingVerification or Rejected status
        match campaign.status {
            CampaignStatus::PendingVerification => {
                // Update fields if provided
                if let Some(new_name) = name {
                    campaign.name = new_name;
                }
                if let Some(new_description) = description {
                    campaign.description = new_description;
                }
                if let Some(new_target) = target_amount {
                    campaign.target_amount = new_target;
                }
                if let Some(new_end_date) = end_date {
                    campaign.end_date = new_end_date;
                }
                if let Some(new_image_url) = image_url {
                    campaign.image_url = Some(new_image_url);
                }
    
                // Update in repository
                self.repository.update_campaign(campaign).await
            },
            _ => Err(AppError::InvalidOperation(format!("Cannot update campaign in {:?} state", campaign.status))),
        }
    }

    pub async fn delete_campaign(
        &self,
        id: i32,
        user_id: i32,
    ) -> Result<(), AppError> {
        // ambil campaign atau 404
        let campaign = self.fetch_or_404(id).await?;
        // cek ownership
        if campaign.user_id != user_id {
            return Err(AppError::InvalidOperation(
                "Not authorized to delete this campaign".into()
            ));
        }
        // cek state
        match campaign.status {
            CampaignStatus::PendingVerification | CampaignStatus::Rejected => {
                // panggil repo, buang hasil boolean
                self.repository.delete_campaign(id).await?;
                Ok(())
            }
            _ => Err(AppError::InvalidOperation(
                format!("Cannot delete campaign in {:?} state", campaign.status)
            )),
        }
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