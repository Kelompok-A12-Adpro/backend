#[cfg(test)]
mod tests {
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use crate::service::campaign::state::campaign_state::{PendingState, ActiveState, CampaignState}; // TAMBAH INI
    use chrono::Utc;

    fn create_test_campaign(status: CampaignStatus) -> Campaign {
        let now = Utc::now();
        Campaign {
            id: 1,
            user_id: 1,
            name: String::from("Test Campaign"),
            description: String::from("Test Description"),
            target_amount: 5000,
            collected_amount: 0,
            start_date: now,
            end_date: now + chrono::Duration::days(30),
            image_url: None,
            status,
            created_at: now,
            updated_at: now,
            evidence_url: None,
            evidence_uploaded_at: None,
        }
    }

    #[test]
    fn test_pending_to_active() {
        let mut campaign = create_test_campaign(CampaignStatus::PendingVerification);
        let pending_state = PendingState {};
        
        let result = pending_state.approve(&mut campaign);
        assert!(result.is_ok());
        assert_eq!(campaign.status, CampaignStatus::Active);
    }

    #[test]
    fn test_pending_to_rejected() {
        let mut campaign = create_test_campaign(CampaignStatus::PendingVerification);
        let pending_state = PendingState {};
        
        let result = pending_state.reject(&mut campaign);
        assert!(result.is_ok());
        assert_eq!(campaign.status, CampaignStatus::Rejected);
    }

    #[test]
    fn test_invalid_transition() {
        let mut campaign = create_test_campaign(CampaignStatus::Active);
        let active_state = ActiveState {};
        
        let result = active_state.reject(&mut campaign);
        assert!(result.is_err());
        assert_eq!(campaign.status, CampaignStatus::Active); // Unchanged
    }
}