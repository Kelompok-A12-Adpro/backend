#[cfg(test)]
mod tests {
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use crate::repository::campaign::campaign_repository::{CampaignRepository, PgCampaignRepository};
    use crate::db::get_test_pool;
    use chrono::Utc;
    use serial_test::serial;

    async fn create_test_repo() -> PgCampaignRepository {
        let pool = get_test_pool().await;
        PgCampaignRepository::new(pool)
    }

    fn create_test_campaign() -> Campaign {
        let now = Utc::now();
        Campaign {
            id: 0,
            user_id: 1,
            name: String::from("Test Campaign"),
            description: String::from("Test Description"),
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
    #[serial]
    async fn test_create_campaign() {
        let repo = create_test_repo().await;
        let campaign = create_test_campaign();
        
        let result = repo.create_campaign(campaign).await;
        assert!(result.is_ok());
        
        let saved = result.unwrap();
        assert!(saved.id > 0);
        assert_eq!(saved.name, "Test Campaign");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_campaign() {
        let repo = create_test_repo().await;
        let campaign = create_test_campaign();
        
        let created = repo.create_campaign(campaign).await.unwrap();
        let result = repo.get_campaign(created.id).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    #[serial]
    async fn test_update_campaign_status() {
        let repo = create_test_repo().await;
        let campaign = create_test_campaign();
        
        let created = repo.create_campaign(campaign).await.unwrap();
        let result = repo.update_campaign_status(created.id, CampaignStatus::Active).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}