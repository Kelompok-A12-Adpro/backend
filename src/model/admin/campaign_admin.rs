use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CampaignVerificationRequest {
    pub approved: bool,
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_verification_request_instantiation() {
        let req = CampaignVerificationRequest {
            approved: true,
            reason: Some("Looks good".to_string()),
        };
        assert!(req.approved);
        assert_eq!(req.reason, Some("Looks good".to_string()));

        let req_no_reason = CampaignVerificationRequest {
            approved: false,
            reason: None,
        };
        assert!(!req_no_reason.approved);
        assert_eq!(req_no_reason.reason, None);
    }

    #[test]
    fn test_campaign_verification_request_serialization_deserialization() {
        let req = CampaignVerificationRequest {
            approved: true,
            reason: Some("Serialization test".to_string()),
        };

        let serialized = serde_json::to_string(&req).expect("Serialization failed");
        let deserialized: CampaignVerificationRequest = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(req.approved, deserialized.approved);
        assert_eq!(req.reason, deserialized.reason);
    }
}
