use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationTargetType {
    AllUsers,
    Fundraisers,
    SpecificUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub target_type: NotificationTargetType,
    pub target_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub content: String,
    pub target_type: NotificationTargetType,
    pub target_id: Option<i32>,
}

pub fn validate_request(req: &CreateNotificationRequest) -> Result<(), String> {
    if req.title.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if req.content.is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    match req.target_type {
        NotificationTargetType::SpecificUser => {
            if req.target_id.is_none() {
                return Err("target_id is required for SpecificUser".to_string());
            }
        }
        NotificationTargetType::AllUsers | NotificationTargetType::Fundraisers => {
            if req.target_id.is_some() {
                return Err("target_id must be None for AllUsers or Fundraisers".to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json;

    #[test]
    fn test_notification_target_type_equality() {
        assert_eq!(
            NotificationTargetType::AllUsers,
            NotificationTargetType::AllUsers
        );
        assert_ne!(
            NotificationTargetType::AllUsers,
            NotificationTargetType::SpecificUser
        );
    }

    #[test]
    fn test_notification_instantiation() {
        let now = Utc::now();
        let notification = Notification {
            id: 1,
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            created_at: now,
            target_type: NotificationTargetType::AllUsers,
        };

        assert_eq!(notification.id, 1);
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.content, "Test Content");
        assert_eq!(notification.created_at, now);
        assert_eq!(notification.target_type, NotificationTargetType::AllUsers);
    }

    #[test]
    fn test_create_notification_request_instantiation() {
        let req_all = CreateNotificationRequest {
            title: "All Users Title".to_string(),
            content: "All Users Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };
        assert_eq!(req_all.title, "All Users Title");
        assert_eq!(req_all.target_type, NotificationTargetType::AllUsers);
        assert_eq!(req_all.adt_detail, None);

        let req_specific = CreateNotificationRequest {
            title: "Specific User Title".to_string(),
            content: "Specific User Content".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            adt_detail: Some("user@example.com".to_string()),
        };
        assert_eq!(req_specific.title, "Specific User Title");
        assert_eq!(
            req_specific.target_type,
            NotificationTargetType::SpecificUser
        );
        assert_eq!(req_specific.adt_detail, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_create_notification_request_serialization_deserialization() {
        let req = CreateNotificationRequest {
            title: "Serialization Test".to_string(),
            content: "Testing Serde".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let serialized = serde_json::to_string(&req).expect("Serialization failed");

        let deserialized: CreateNotificationRequest =
            serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(req.title, deserialized.title);
        assert_eq!(req.content, deserialized.content);
        assert_eq!(req.target_type, deserialized.target_type);
        assert_eq!(req.adt_detail, deserialized.adt_detail);

        // Test with adt_detail
        let req_with_detail = CreateNotificationRequest {
            title: "Test with Detail".to_string(),
            content: "Testing with adt_detail".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            adt_detail: Some("test@example.com".to_string()),
        };

        let serialized_with_detail = serde_json::to_string(&req_with_detail).expect("Serialization failed");
        let deserialized_with_detail: CreateNotificationRequest =
            serde_json::from_str(&serialized_with_detail).expect("Deserialization failed");
        assert_eq!(req_with_detail.adt_detail, deserialized_with_detail.adt_detail);
    }

    #[test]
    fn test_create_notification_request_validation() {
        // Valid: AllUsers without adt_detail
        let valid_all_users = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };
        assert!(validate_request(&valid_all_users).is_ok());

        // Invalid: Empty title
        let invalid_empty_title = CreateNotificationRequest {
            title: "".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };
        assert!(validate_request(&invalid_empty_title).is_err());

        // Invalid: Empty content
        let invalid_empty_content = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };
        assert!(validate_request(&invalid_empty_content).is_err());

        // Valid: SpecificUser with valid email
        let valid_specific_user = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            adt_detail: Some("user@example.com".to_string()),
        };
        assert!(validate_request(&valid_specific_user).is_ok());

        // Invalid: SpecificUser without adt_detail
        let invalid_specific_user_no_detail = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            adt_detail: None,
        };
        assert!(validate_request(&invalid_specific_user_no_detail).is_err());

        // Invalid: SpecificUser with invalid email
        let invalid_specific_user_bad_email = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            adt_detail: Some("invalid-email".to_string()),
        };
        assert!(validate_request(&invalid_specific_user_bad_email).is_err());

        // Valid: Fundraisers with valid campaign ID
        let valid_fundraisers = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::Fundraisers,
            adt_detail: Some("123".to_string()),
        };
        assert!(validate_request(&valid_fundraisers).is_ok());

        // Invalid: Fundraisers with invalid campaign ID
        let invalid_fundraisers_bad_id = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::Fundraisers,
            adt_detail: Some("not-a-number".to_string()),
        };
        assert!(validate_request(&invalid_fundraisers_bad_id).is_err());

        // Valid: Donors with valid campaign ID
        let valid_donors = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::Donors,
            adt_detail: Some("456".to_string()),
        };
        assert!(validate_request(&valid_donors).is_ok());

        // Valid: NewCampaign with valid email
        let valid_new_campaign = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::NewCampaign,
            adt_detail: Some("owner@example.com".to_string()),
        };
        assert!(validate_request(&valid_new_campaign).is_ok());
    }
}
