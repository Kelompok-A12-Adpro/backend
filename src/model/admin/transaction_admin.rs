use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionFilterRequest {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub campaign_id: Option<i32>,
    pub status: String, // "Pending", "Completed", "Failed"
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json;

    #[test]
    fn test_transaction_filter_request_instantiation() {
        let now = Utc::now();
        let later = now + chrono::Duration::days(1);

        // All fields Some
        let req_full = TransactionFilterRequest {
            start_date: Some(now),
            end_date: Some(later),
            campaign_id: Some(123),
            status: "Completed".to_string(),
        };
        assert_eq!(req_full.start_date, Some(now));
        assert_eq!(req_full.end_date, Some(later));
        assert_eq!(req_full.campaign_id, Some(123));
        assert_eq!(req_full.status, "Completed");

        // Some fields None
        let req_partial = TransactionFilterRequest {
            start_date: None,
            end_date: None,
            campaign_id: Some(456),
            status: "Pending".to_string(),
        };
        assert!(req_partial.start_date.is_none());
        assert!(req_partial.end_date.is_none());
        assert_eq!(req_partial.campaign_id, Some(456));
        assert_eq!(req_partial.status, "Pending");
    }

    #[test]
    fn test_transaction_filter_request_serialization_deserialization() {
        // Using a fixed date for reliable serialization comparison
        let fixed_date = Utc.with_ymd_and_hms(2024, 5, 1, 10, 0, 0).unwrap();

        let req = TransactionFilterRequest {
            start_date: Some(fixed_date),
            end_date: None,
            campaign_id: Some(789),
            status: "Failed".to_string(),
        };

        let serialized = serde_json::to_string(&req).expect("Serialization failed");
        let deserialized: TransactionFilterRequest = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(req.start_date, deserialized.start_date);
        assert_eq!(req.end_date, deserialized.end_date);
        assert_eq!(req.campaign_id, deserialized.campaign_id);
        assert_eq!(req.status, deserialized.status);
    }
}
