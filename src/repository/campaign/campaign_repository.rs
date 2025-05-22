use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::Utc;

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

pub struct InMemoryCampaignRepository {
    campaigns: Arc<Mutex<HashMap<i32, Campaign>>>,
    next_id: Arc<Mutex<i32>>,
}

impl InMemoryCampaignRepository {
    pub fn new() -> Self {
        InMemoryCampaignRepository {
            campaigns: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
}

#[async_trait]
impl CampaignRepository for InMemoryCampaignRepository {
    async fn create_campaign(&self, mut campaign: Campaign) -> Result<Campaign, AppError> {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        
        campaign.id = id;
        campaign.created_at = Utc::now();
        campaign.updated_at = Utc::now();
        
        let mut campaigns = self.campaigns.lock().unwrap();
        campaigns.insert(id, campaign.clone());
        
        Ok(campaign)
    }

    async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError> {
        let campaigns = self.campaigns.lock().unwrap();
        if let Some(campaign) = campaigns.get(&id) {
            Ok(Some(campaign.clone()))
        } else {
            Err(AppError::NotFound(format!("Campaign with id {} not found", id)))
        }
    }

    async fn update_campaign(&self, mut campaign: Campaign) -> Result<Campaign, AppError> {
        let mut campaigns = self.campaigns.lock().unwrap();
        
        if !campaigns.contains_key(&campaign.id) {
            return Err(AppError::NotFound(format!("Campaign with id {} not found", campaign.id)));
        }
        
        campaign.updated_at = Utc::now();
        campaigns.insert(campaign.id, campaign.clone());
        
        Ok(campaign)
    } 

    async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError> {
        let mut campaigns = self.campaigns.lock().unwrap();
        
        if let Some(mut campaign) = campaigns.get(&id).cloned() {
            campaign.status = status;
            campaign.updated_at = Utc::now();
            campaigns.insert(id, campaign);
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("Campaign with id {} not found", id)))
        }
    }

    async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
        let campaigns = self.campaigns.lock().unwrap();
        
        let user_campaigns = campaigns
            .values()
            .filter(|campaign| campaign.user_id == user_id)
            .cloned()
            .collect();
        
        Ok(user_campaigns)
    }

    async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError> {
        let campaigns = self.campaigns.lock().unwrap();
        
        let filtered_campaigns = campaigns
            .values()
            .filter(|campaign| campaign.status == status)
            .cloned()
            .collect();
        
        Ok(filtered_campaigns)
    }

    async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError> {
        let campaigns = self.campaigns.lock().unwrap();
        
        let all_campaigns = campaigns.values().cloned().collect();
        
        Ok(all_campaigns)
    }

    async fn delete_campaign(&self, id: i32) -> Result<bool, AppError> {
        let mut campaigns = self.campaigns.lock().unwrap();
        
        if let Some(campaign) = campaigns.get(&id) {
            if campaign.status == CampaignStatus::PendingVerification || campaign.status == CampaignStatus::Rejected {
                campaigns.remove(&id);
                Ok(true)
            } else {
                Err(AppError::InvalidOperation(format!("Cannot delete campaign in {:?} state", campaign.status)))
            }
        } else {
            Err(AppError::NotFound(format!("Campaign with id {} not found", id)))
        }
    }

}