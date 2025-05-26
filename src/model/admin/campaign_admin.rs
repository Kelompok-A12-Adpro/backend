use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CampaignVerificationDetail {
    pub campaign_id: i32,
    pub approved: bool,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_verification_request_instantiation() {
        let req = CampaignVerificationDetail {
            campaign_id: 1,
            approved: true,
            notes: Some("Looks good".to_string()),
        };
        assert!(req.approved);
        assert_eq!(req.notes, Some("Looks good".to_string()));

        let req_no_notes = CampaignVerificationDetail {
            campaign_id: 2,
            approved: false,
            notes: None,
        };
        assert!(!req_no_notes.approved);
        assert_eq!(req_no_notes.notes, None);
    }

    #[test]
    fn test_campaign_verification_request_serialization_deserialization() {
        let req = CampaignVerificationDetail {
            campaign_id: 3,
            approved: true,
            notes: Some("Serialization test".to_string()),
        };

        let serialized = serde_json::to_string(&req).expect("Serialization failed");
        let deserialized: CampaignVerificationDetail = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(req.approved, deserialized.approved);
        assert_eq!(req.notes, deserialized.notes);
    }
}
