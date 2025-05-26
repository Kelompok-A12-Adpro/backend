use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FundUsageDetail {
    pub campaign_id: i32,
    pub approve: bool,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fund_usage_verification_request_instantiation() {
        let req_approve = FundUsageDetail {
            campaign_id: 1,
            approve: true,
            notes: Some("Looks valid".to_string()),
        };
        assert!(req_approve.approve);
        assert_eq!(req_approve.notes, Some("Looks valid".to_string()));

        let req_reject = FundUsageDetail {
            campaign_id: 2,
            approve: false,
            notes: Some("Missing details".to_string()),
        };
        assert!(!req_reject.approve);
        assert_eq!(req_reject.notes, Some("Missing details".to_string()));

         let req_no_notes = FundUsageDetail {
            campaign_id: 3,
            approve: true,
            notes: None,
        };
        assert!(req_no_notes.approve);
        assert!(req_no_notes.notes.is_none());
    }

    #[test]
    fn test_fund_usage_verification_request_serialization_deserialization() {
        let req = FundUsageDetail {
            campaign_id: 4,
            approve: true,
            notes: Some("Serialization test".to_string()),
        };

        let serialized = serde_json::to_string(&req).expect("Serialization failed");
        let deserialized: FundUsageDetail = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(req.approve, deserialized.approve);
        assert_eq!(req.notes, deserialized.notes);
    }
}
