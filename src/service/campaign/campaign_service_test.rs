#[cfg(test)]
mod tests {
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use crate::repository::campaign::campaign_repository::CampaignRepository;
    use crate::service::campaign::factory::campaign_factory::CampaignFactory;
    use crate::service::campaign::campaign_service::CampaignService; // TAMBAH INI
    use crate::errors::AppError;
    use async_trait::async_trait;
    use std::sync::Arc;
    use chrono::Utc;
    use mockall::mock;

    mock! {
        TestCampaignRepository {}
        
        #[async_trait]
        impl CampaignRepository for TestCampaignRepository {
            async fn create_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>;
            async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError>;
            async fn update_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>;
            async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError>;
            async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError>;
            async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError>;
            async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError>;
            async fn delete_campaign(&self, id: i32) -> Result<bool, AppError>;
        }
    }

    fn create_test_campaign() -> Campaign {
        let now = Utc::now();
        Campaign {
            id: 1,
            user_id: 1,
            name: "Test Campaign".to_string(),
            description: "Test Description".to_string(),
            target_amount: 5000,
            collected_amount: 0,
            start_date: now,
            end_date: now + chrono::Duration::days(30),
            image_url: None,
            status: CampaignStatus::PendingVerification,
            created_at: now,
            updated_at: now,
            evidence_url: None,
            evidence_uploaded_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_campaign_success() {
        let mut mock_repo = MockTestCampaignRepository::new();
        let factory = Arc::new(CampaignFactory::new());
        
        let expected = create_test_campaign();
        let expected_clone = expected.clone();
        
        mock_repo
            .expect_create_campaign()
            .returning(move |_| Ok(expected_clone.clone()));

        let service = CampaignService::new(Arc::new(mock_repo), factory);
        
        let result = service.create_campaign(
            1,
            "Test Campaign".to_string(),
            "Test Description".to_string(),
            5000,
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
            None,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_campaign_success() {
        let mut mock_repo = MockTestCampaignRepository::new();
        let factory = Arc::new(CampaignFactory::new());
        
        let expected = create_test_campaign();
        let expected_clone = expected.clone();
        
        mock_repo
            .expect_get_campaign()
            .returning(move |_| Ok(Some(expected_clone.clone())));

        let service = CampaignService::new(Arc::new(mock_repo), factory);
        let result = service.get_campaign(1).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_campaign_not_found() {
        let mut mock_repo = MockTestCampaignRepository::new();
        let factory = Arc::new(CampaignFactory::new());
        
        mock_repo
            .expect_get_campaign()
            .returning(move |_| Ok(None));

        let service = CampaignService::new(Arc::new(mock_repo), factory);
        let result = service.get_campaign(999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_campaign_success() {
        let mut mock_repo = MockTestCampaignRepository::new();
        let factory = Arc::new(CampaignFactory::new());
        
        let campaign = create_test_campaign();
        let campaign_clone = campaign.clone();
        
        mock_repo
            .expect_get_campaign()
            .returning(move |_| Ok(Some(campaign_clone.clone())));
            
        mock_repo
            .expect_delete_campaign()
            .returning(move |_| Ok(true));

        let service = CampaignService::new(Arc::new(mock_repo), factory);
        let result = service.delete_campaign(1, 1).await;

        assert!(result.is_ok());
    }
}