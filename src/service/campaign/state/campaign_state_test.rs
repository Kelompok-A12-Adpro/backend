#[cfg(test)]
mod tests {
    use crate::service::campaign::state::campaign_state::{CampaignState, PendingState, ActiveState, RejectedState};
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use chrono::Utc;

    #[test]
    fn test_pending_to_active_transition() {
        let mut campaign = Campaign {
            id: 1,
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
        
        let pending_state = PendingState {};
        let new_state = pending_state.approve(&mut campaign);
        
        assert!(new_state.is_ok());
        assert_eq!(campaign.status, CampaignStatus::Active);
    }

    #[test]
    fn test_pending_to_rejected_transition() {
        let mut campaign = Campaign {
            id: 1,
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
        
        let pending_state = PendingState {};
        let new_state = pending_state.reject(&mut campaign);
        
        assert!(new_state.is_ok());
        assert_eq!(campaign.status, CampaignStatus::Rejected);
    }

    #[test]
    fn test_invalid_transition() {
        let mut campaign = Campaign {
            id: 1,
            user_id: 1,
            name: String::from("Test Campaign"),
            description: String::from("Test Description"),
            target_amount: 5000.0,
            collected_amount: 0.0,
            status: CampaignStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        };
        
        let active_state = ActiveState {};
        let result = active_state.reject(&mut campaign);
        
        assert!(result.is_err());
        assert_eq!(campaign.status, CampaignStatus::Active); // Unchanged
    }
}