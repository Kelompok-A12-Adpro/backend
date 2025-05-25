use chrono::{DateTime, Utc};
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
        target_amount: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        image_url: Option<String>,
    ) -> Campaign {
        let now = Utc::now();
        
        Campaign {
            id: 0, // Will be set by the repository
            user_id,
            name,
            description,
            target_amount,
            collected_amount: 0,
            status: CampaignStatus::PendingVerification,
            start_date,
            end_date,
            image_url,
            created_at: now,
            updated_at: now,
            evidence_url: None,
            evidence_uploaded_at: None,
        }
    }
}