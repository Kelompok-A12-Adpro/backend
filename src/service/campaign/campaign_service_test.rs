#[cfg(test)]
mod tests {
    use crate::service::campaign::campaign_service::CampaignService;
    use crate::service::campaign::factory::campaign_factory::CampaignFactory;
    use crate::repository::campaign::campaign_repository::{CampaignRepository, InMemoryCampaignRepository};
    use crate::service::campaign::observer::campaign_observer::CampaignNotifier;
    use crate::model::campaign::campaign::CampaignStatus;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_campaign() {
        let repo = Arc::new(InMemoryCampaignRepository::new());
        let factory = Arc::new(CampaignFactory::new());
        let notifier = Arc::new(CampaignNotifier::new());
        
        let service = CampaignService::new(repo, factory, notifier);
        
        let result = service.create_campaign(
            10,
            "Test Campaign".to_string(),
            "Test Description".to_string(),
            5000.0,
        ).await;
        
        assert!(result.is_ok());
        let campaign = result.unwrap();
        
        assert_eq!(campaign.user_id, 10);
        assert_eq!(campaign.name, "Test Campaign");
        assert_eq!(campaign.target_amount, 5000.0);
        assert_eq!(campaign.status, CampaignStatus::PendingVerification);
    }

    #[tokio::test]
    async fn test_get_campaign() {
        let repo = Arc::new(InMemoryCampaignRepository::new());
        let factory = Arc::new(CampaignFactory::new());
        let notifier = Arc::new(CampaignNotifier::new());
        
        let service = CampaignService::new(repo.clone(), factory, notifier);
        
        // Create test campaign
        let created = service.create_campaign(
            10,
            "Test Campaign".to_string(),
            "Test Description".to_string(),
            5000.0,
        ).await.unwrap();
        
        // Test retrieval
        let result = service.get_campaign(created.id).await;
        
        assert!(result.is_ok());
        let campaign = result.unwrap();
        
        assert!(campaign.is_some());
        let campaign = campaign.unwrap();
        assert_eq!(campaign.id, created.id);
        assert_eq!(campaign.name, "Test Campaign");
    }

    #[tokio::test]
    async fn test_approve_campaign() {
        let repo = Arc::new(InMemoryCampaignRepository::new());
        let factory = Arc::new(CampaignFactory::new());
        let notifier = Arc::new(CampaignNotifier::new());
        
        let service = CampaignService::new(repo.clone(), factory, notifier);
        
        // Create test campaign
        let created = service.create_campaign(
            10,
            "Test Campaign".to_string(),
            "Test Description".to_string(),
            5000.0,
        ).await.unwrap();
        
        // Approve campaign
        let result = service.approve_campaign(created.id).await;
        
        assert!(result.is_ok());
        
        // Verify status was updated
        let updated = service.get_campaign(created.id).await.unwrap().unwrap();
        assert_eq!(updated.status, CampaignStatus::Active);
    }
}