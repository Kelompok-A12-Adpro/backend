use chrono::{DateTime, Utc, TimeZone};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, PartialEq)]
pub struct Donation {
    pub id: i32,
    pub user_id: i32,
    pub campaign_id: i32,
    pub amount: f64,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct NewDonationRequest {
   pub campaign_id: i32,
   pub amount: f64,
   pub message: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UpdateDonationMessageRequest {
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn fixed_utc_datetime() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 10, 26, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_donation_serialization_with_message() {
        let donation = Donation {
            id: 1,
            user_id: 101,
            campaign_id: 202,
            amount: 50.75,
            message: Some("Thank you!".to_string()),
            created_at: fixed_utc_datetime(),
        };

        let serialized_json = serde_json::to_string(&donation).unwrap();
        
        let expected_json = r#"{"id":1,"user_id":101,"campaign_id":202,"amount":50.75,"message":"Thank you!","created_at":"2023-10-26T12:00:00Z"}"#;
        
        assert_eq!(serialized_json, expected_json);
    }

    #[test]
    fn test_donation_serialization_without_message() {
        let donation = Donation {
            id: 2,
            user_id: 102,
            campaign_id: 203,
            amount: 100.0,
            message: None,
            created_at: fixed_utc_datetime(),
        };

        let serialized_json = serde_json::to_string(&donation).unwrap();
        let expected_json = r#"{"id":2,"user_id":102,"campaign_id":203,"amount":100.0,"message":null,"created_at":"2023-10-26T12:00:00Z"}"#;
        
        assert_eq!(serialized_json, expected_json);
    }

    #[test]
    fn test_new_donation_request_deserialization_with_message() {
        let json_input = r#"{
            "campaign_id": 301,
            "amount": 25.50,
            "message": "Keep up the good work!"
        }"#;

        let deserialized_request: NewDonationRequest = serde_json::from_str(json_input).unwrap();

        let expected_request = NewDonationRequest {
            campaign_id: 301,
            amount: 25.50,
            message: Some("Keep up the good work!".to_string()),
        };
        
        assert_eq!(deserialized_request, expected_request);
    }

    #[test]
    fn test_new_donation_request_deserialization_without_message_field() {
        let json_input = r#"{
            "campaign_id": 302,
            "amount": 75.00
        }"#;

        let deserialized_request: NewDonationRequest = serde_json::from_str(json_input).unwrap();

        let expected_request = NewDonationRequest {
            campaign_id: 302,
            amount: 75.00,
            message: None,
        };
        
        assert_eq!(deserialized_request, expected_request);
    }

    #[test]
    fn test_new_donation_request_deserialization_with_null_message() {
        let json_input = r#"{
            "campaign_id": 303,
            "amount": 10.0,
            "message": null
        }"#;

        let deserialized_request: NewDonationRequest = serde_json::from_str(json_input).unwrap();

        let expected_request = NewDonationRequest {
            campaign_id: 303,
            amount: 10.0,
            message: None,
        };
        
        assert_eq!(deserialized_request, expected_request);
    }

    #[test]
    fn test_update_donation_message_request_deserialization_with_message() {
        let json_input = r#"{"message": "Updated message"}"#;
        let deserialized_request: UpdateDonationMessageRequest = serde_json::from_str(json_input).unwrap();
        let expected_request = UpdateDonationMessageRequest {
            message: Some("Updated message".to_string()),
        };
        assert_eq!(deserialized_request, expected_request);
    }

    #[test]
    fn test_update_donation_message_request_deserialization_without_message_field() {
        let json_input = r#"{}"#;
        let deserialized_request: UpdateDonationMessageRequest = serde_json::from_str(json_input).unwrap();
        let expected_request = UpdateDonationMessageRequest {
            message: None,
        };
        assert_eq!(deserialized_request, expected_request);
    }
    
    #[test]
    fn test_update_donation_message_request_deserialization_with_null_message() {
        let json_input = r#"{"message": null}"#;
        let deserialized_request: UpdateDonationMessageRequest = serde_json::from_str(json_input).unwrap();
        let expected_request = UpdateDonationMessageRequest {
            message: None,
        };
        assert_eq!(deserialized_request, expected_request);
    }
}