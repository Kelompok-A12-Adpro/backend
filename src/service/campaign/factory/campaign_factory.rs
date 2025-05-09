use chrono::Utc;
use crate::model::campaign::campaign::{Campaign, CampaignStatus};

pub struct CampaignFactory;

impl CampaignFactory {
    pub fn new() -> Self {
        CampaignFactory
    }
    
    pub fn create_campaign(
        &self,
        user_id: i32,
        name: String,
        description: String,
        target_amount: f64,
    ) -> Campaign {
        let now = Utc::now();
        
        Campaign {
            id: 0, // Will be set by the repository
            user_id,
            name,
            description,
            target_amount,
            collected_amount: 0.0,
            status: CampaignStatus::PendingVerification,
            created_at: now,
            updated_at: now,
            evidence_url: None,
            evidence_uploaded_at: None,
        }
    }
}