use std::sync::Arc;
use rocket::State;

use crate::model::admin::notification::CreateNotificationRequest;
use crate::model::campaign::campaign::{Campaign, CampaignStatus};
use crate::repository::campaign::campaign_repository::CampaignRepository;
use crate::service::admin::notification::notification_service::{self, NotificationService};
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
        match self.repository.get_campaigns_by_user(user_id).await {
            Ok(campaigns) => Ok(campaigns),
            Err(AppError::NotFound(_)) => Ok(Vec::new()), // Return empty vector if user has no campaigns
            Err(e) => Err(e), // Propagate other errors
        }
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

     pub async fn approve_campaign(
        &self, id: i32,
        notification_service: &State<Arc<NotificationService>>,
    ) -> Result<Campaign, AppError> {
        let mut campaign = self.fetch_or_404(id).await?;
        let state = Self::state_from_status(campaign.status.clone());
        state.approve(&mut campaign)?;

        let updated = self.repository.update_campaign(campaign).await?;

        // Create notification for subscriber
        let notification_service_clone = Arc::clone(notification_service);
        let campaign_name = updated.name.clone();
        tokio::spawn(async move {
            if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                title: "New Campaign Available".to_string(),
                content: format!("A new campaign '{}' has been created recently!", campaign_name),
                target_type: crate::model::admin::notification::NotificationTargetType::NewCampaign,
                adt_detail: None,
            }).await {
                eprintln!("Failed to send notification: {:?}", e);
            }

            if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                title: "Campaign Accepted".to_string(),
                content: format!("Your campaign '{}' has been approved and is now active!", campaign_name),
                target_type: crate::model::admin::notification::NotificationTargetType::Fundraisers,
                adt_detail: Some(id.clone()),
            }).await {
                eprintln!("Failed to send notification: {:?}", e);
            }
        });
        
        Ok(updated)
    }

    pub async fn reject_campaign(
        &self,
        id: i32,
        reason: Option<String>,
        notification_service: &State<Arc<NotificationService>>,
    ) -> Result<Campaign, AppError> {
        let mut campaign = self.fetch_or_404(id).await?;
        let state = Self::state_from_status(campaign.status.clone());
        state.reject(&mut campaign)?;

        let updated = self.repository.update_campaign(campaign).await?;

        // Create notification for fundraiser
        let notification_service_clone = Arc::clone(notification_service);
        let campaign_name = updated.name.clone();
        let reject_reason = reason.clone();
        tokio::spawn(async move {
            if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                title: "Campaign Rejected".to_string(),
                content: format!("Your campaign '{}' has been rejected. Reason: {}", 
                    campaign_name, reject_reason.unwrap_or("No reason provided".to_string())),
                target_type: crate::model::admin::notification::NotificationTargetType::Fundraisers,
                adt_detail: Some(id.clone()),
            }).await {
                eprintln!("Failed to send notification: {:?}", e);
            }
        });

        Ok(updated)
    }

    pub async fn complete_campaign(&self, id: i32, notification_service: &State<Arc<NotificationService>>) -> Result<Campaign, AppError> {
        let mut campaign = self.fetch_or_404(id).await?;
        let state = Self::state_from_status(campaign.status.clone());
        state.complete(&mut campaign)?;

        let updated = self.repository.update_campaign(campaign).await?;

        let notification_service_clone = Arc::clone(notification_service);
        let campaign_name = updated.name.clone();
        let campaign_name2 = updated.name.clone();
        tokio::spawn(async move {
            // Create notification for fundraiser
            if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                title: "Campaign Completed".to_string(),
                content: format!("Your campaign '{}' has been completed successfully!", campaign_name),
                target_type: crate::model::admin::notification::NotificationTargetType::Fundraisers,
                adt_detail: Some(updated.id),
            }).await {
                eprintln!("Failed to send notification: {:?}", e);
            }

            // Create notification for donors
            if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                title: "Campaign Target Reached".to_string(),
                content: format!("The campaign '{}' you donated before has reached its target amount of {}. Thank you for your support!", 
                    campaign_name2, updated.target_amount),
                target_type: crate::model::admin::notification::NotificationTargetType::Donors,
                adt_detail: None,
            }).await {
                eprintln!("Failed to send notification: {:?}", e);
            }
        });

        Ok(updated)
    }
    
    // TODO: Add more methods if needed, such as for handling donations, evidence uploads, etc.
}