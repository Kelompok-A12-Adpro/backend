#[cfg(test)]
mod tests {
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use crate::repository::campaign::campaign_repository::{CampaignRepository, InMemoryCampaignRepository};
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_campaign() {
        let repo = InMemoryCampaignRepository::new();
        
        let campaign = Campaign {
            id: 0, // Will be assigned by repo
            user_id: 1,
            name: String::from("Test Campaign"),
            description: String::from("Test Description"),
            target_amount: 5000.0,
            collected_amount: 0.0,
            status: CampaignStatus::PendingVerification,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        };
        
        let result = repo.create_campaign(campaign).await;
        assert!(result.is_ok());
        
        let saved_campaign = result.unwrap();
        assert!(saved_campaign.id > 0);
        assert_eq!(saved_campaign.name, "Test Campaign");
    }

    #[tokio::test]
    async fn test_get_campaign() {
        let repo = InMemoryCampaignRepository::new();
        
        // Create test campaign
        let campaign = Campaign {
            id: 0,
            user_id: 1,
            name: String::from("Test Campaign"),
            description: String::from("Test Description"),
            target_amount: 5000.0,
            collected_amount: 0.0,
            status: CampaignStatus::PendingVerification,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        };
        
        let created = repo.create_campaign(campaign).await.unwrap();
        
        // Test retrieval
        let result = repo.get_campaign(created.id).await;
        assert!(result.is_ok());
        
        let retrieved = result.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_update_campaign_status() {
        let repo = InMemoryCampaignRepository::new();
        
        // Create test campaign
        let campaign = Campaign {
            id: 0,
            user_id: 1,
            name: String::from("Test Campaign"),
            description: String::from("Test Description"),
            target_amount: 5000.0,
            collected_amount: 0.0,
            status: CampaignStatus::PendingVerification,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        };
        
        let created = repo.create_campaign(campaign).await.unwrap();
        
        // Test status update
        let result = repo.update_campaign_status(created.id, CampaignStatus::Active).await;
        assert!(result.is_ok());
        
        // Verify status was updated
        let updated = repo.get_campaign(created.id).await.unwrap().unwrap();
        assert_eq!(updated.status, CampaignStatus::Active);
    }
}