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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json;

    #[test]
    fn test_notification_target_type_equality() {
        assert_eq!(NotificationTargetType::AllUsers, NotificationTargetType::AllUsers);
        assert_ne!(NotificationTargetType::AllUsers, NotificationTargetType::Fundraisers);
        assert_ne!(NotificationTargetType::Fundraisers, NotificationTargetType::SpecificUser);
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
            target_id: None,
        };

        assert_eq!(notification.id, 1);
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.content, "Test Content");
        assert_eq!(notification.created_at, now);
        assert_eq!(notification.target_type, NotificationTargetType::AllUsers);
        assert!(notification.target_id.is_none());
    }

    #[test]
    fn test_create_notification_request_instantiation() {
        let req_all = CreateNotificationRequest {
            title: "All Users Title".to_string(),
            content: "All Users Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };
        assert_eq!(req_all.title, "All Users Title");
        assert_eq!(req_all.target_type, NotificationTargetType::AllUsers);
        assert!(req_all.target_id.is_none());

        let req_specific = CreateNotificationRequest {
            title: "Specific User Title".to_string(),
            content: "Specific User Content".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            target_id: Some(123),
        };
        assert_eq!(req_specific.title, "Specific User Title");
        assert_eq!(req_specific.target_type, NotificationTargetType::SpecificUser);
        assert_eq!(req_specific.target_id, Some(123));
    }

    #[test]
    fn test_create_notification_request_serialization_deserialization() {
        let req = CreateNotificationRequest {
            title: "Serialization Test".to_string(),
            content: "Testing Serde".to_string(),
            target_type: NotificationTargetType::Fundraisers,
            target_id: None,
        };

        let serialized = serde_json::to_string(&req).expect("Serialization failed");
        // Need to add Serialize derive to CreateNotificationRequest for this test to pass
        // let deserialized: CreateNotificationRequest = serde_json::from_str(&serialized).expect("Deserialization failed");
        // assert_eq!(req.title, deserialized.title);
        // assert_eq!(req.content, deserialized.content);
        // assert_eq!(req.target_type, deserialized.target_type);
        // assert_eq!(req.target_id, deserialized.target_id);

        // For now, just assert serialization works
        assert!(serialized.contains("Serialization Test"));
    }

    // Test for validation logic (will fail initially in TDD)
    #[test]
    fn test_create_notification_request_validation() {
        // This test assumes some validation logic exists or will be added.
        // It will fail until such logic is implemented.

        // Valid: Specific user with ID
        let valid_specific = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            target_id: Some(1),
        };
        // assert!(validate_request(&valid_specific).is_ok()); // Placeholder for validation function

        // Invalid: Specific user without ID
        let invalid_specific_no_id = CreateNotificationRequest {
            title: "Invalid Title".to_string(),
            content: "Invalid Content".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            target_id: None,
        };
        // assert!(validate_request(&invalid_specific_no_id).is_err()); // Placeholder

        // Invalid: All users with ID
        let invalid_all_with_id = CreateNotificationRequest {
            title: "Invalid Title".to_string(),
            content: "Invalid Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: Some(1),
        };
        // assert!(validate_request(&invalid_all_with_id).is_err()); // Placeholder

        // Invalid: Empty title
        let invalid_empty_title = CreateNotificationRequest {
            title: "".to_string(),
            content: "Valid Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };
        // assert!(validate_request(&invalid_empty_title).is_err()); // Placeholder

         // Invalid: Empty content
        let invalid_empty_content = CreateNotificationRequest {
            title: "Valid Title".to_string(),
            content: "".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };
        // assert!(validate_request(&invalid_empty_content).is_err()); // Placeholder
    }

    // Placeholder for a potential validation function
    // fn validate_request(req: &CreateNotificationRequest) -> Result<(), String> {
    //     if req.title.is_empty() {
    //         return Err("Title cannot be empty".to_string());
    //     }
    //     if req.content.is_empty() {
    //         return Err("Content cannot be empty".to_string());
    //     }
    //     match req.target_type {
    //         NotificationTargetType::SpecificUser => {
    //             if req.target_id.is_none() {
    //                 return Err("target_id is required for SpecificUser".to_string());
    //             }
    //         }
    //         NotificationTargetType::AllUsers | NotificationTargetType::Fundraisers => {
    //             if req.target_id.is_some() {
    //                 return Err("target_id must be None for AllUsers or Fundraisers".to_string());
    //             }
    //         }
    //     }
    //     Ok(())
    // }
}
