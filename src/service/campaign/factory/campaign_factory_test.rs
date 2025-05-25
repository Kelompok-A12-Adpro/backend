#[cfg(test)]
mod tests {
    use crate::service::campaign::factory::campaign_factory::CampaignFactory;
    use crate::model::campaign::campaign::CampaignStatus;
    use chrono::Utc;

    #[test]
    fn test_create_campaign() {
        let factory = CampaignFactory::new();
        let now = Utc::now();
        
        let campaign = factory.create_campaign(
            10,
            "Test Campaign".to_string(),
            "Test Description".to_string(),
            5000,
            now,
            now + chrono::Duration::days(30),
            None,
        );
        
        assert_eq!(campaign.user_id, 10);
        assert_eq!(campaign.name, "Test Campaign");
        assert_eq!(campaign.target_amount, 5000);
        assert_eq!(campaign.collected_amount, 0);
        assert_eq!(campaign.status, CampaignStatus::PendingVerification);
        assert!(campaign.created_at >= now);
    }
}