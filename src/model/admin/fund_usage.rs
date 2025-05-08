use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FundUsageVerificationRequest {
    pub approve: bool,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fund_usage_verification_request_instantiation() {
        let req_approve = FundUsageVerificationRequest {
            approve: true,
            notes: Some("Looks valid".to_string()),
        };
        assert!(req_approve.approve);
        assert_eq!(req_approve.notes, Some("Looks valid".to_string()));

        let req_reject = FundUsageVerificationRequest {
            approve: false,
            notes: Some("Missing details".to_string()),
        };
        assert!(!req_reject.approve);
        assert_eq!(req_reject.notes, Some("Missing details".to_string()));

         let req_no_notes = FundUsageVerificationRequest {
            approve: true,
            notes: None,
        };
        assert!(req_no_notes.approve);
        assert!(req_no_notes.notes.is_none());
    }

    #[test]
    fn test_fund_usage_verification_request_serialization_deserialization() {
        let req = FundUsageVerificationRequest {
            approve: true,
            notes: Some("Serialization test".to_string()),
        };

        let serialized = serde_json::to_string(&req).expect("Serialization failed");
        let deserialized: FundUsageVerificationRequest = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(req.approve, deserialized.approve);
        assert_eq!(req.notes, deserialized.notes);
    }
}
