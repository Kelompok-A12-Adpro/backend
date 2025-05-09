#[cfg(test)]
mod tests {
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use chrono::Utc;

    #[test]
    fn test_create_campaign() {
        let campaign = Campaign {
            id: 1,
            user_id: 10,
            name: String::from("Save the Oceans"),
            description: String::from("Campaign to clean oceans"),
            target_amount: 10000.0,
            collected_amount: 0.0,
            status: CampaignStatus::PendingVerification,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        };

        assert_eq!(campaign.id, 1);
        assert_eq!(campaign.user_id, 10);
        assert_eq!(campaign.name, "Save the Oceans");
        assert_eq!(campaign.target_amount, 10000.0);
        assert_eq!(campaign.status, CampaignStatus::PendingVerification);
    }

    #[test]
    fn test_campaign_status_transition() {
        let mut campaign = Campaign {
            id: 1,
            user_id: 10,
            name: String::from("Save the Oceans"),
            description: String::from("Campaign to clean oceans"),
            target_amount: 10000.0,
            collected_amount: 0.0,
            status: CampaignStatus::PendingVerification,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        };

        // Test initial status
        assert_eq!(campaign.status, CampaignStatus::PendingVerification);
        
        // Test transition to Active
        campaign.status = CampaignStatus::Active;
        assert_eq!(campaign.status, CampaignStatus::Active);
    }
}