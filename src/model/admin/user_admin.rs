use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Blocked,
}

#[derive(Debug, Deserialize)]
pub struct UserActionRequest {
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_user_status_equality() {
        assert_eq!(UserStatus::Active, UserStatus::Active);
        assert_ne!(UserStatus::Active, UserStatus::Blocked);
    }

    #[test]
    fn test_user_action_request_instantiation() {
        let req = UserActionRequest {
            reason: "Violation of terms".to_string(),
        };
        assert_eq!(req.reason, "Violation of terms");
    }

    #[test]
    fn test_user_action_request_deserialization() {
        let json_data = r#"{
            "reason": "Suspicious activity detected"
        }"#;

        let deserialized: UserActionRequest = serde_json::from_str(json_data).expect("Deserialization failed");

        assert_eq!(deserialized.reason, "Suspicious activity detected");
    }

    // NOTE: Test serialization if needed later
    // #[test]
    // fn test_user_action_request_serialization() {
    //     // Add #[derive(Serialize)] to UserActionRequest for this test
    //     let req = UserActionRequest {
    //         reason: "Testing serialization".to_string(),
    //     };
    //     let serialized = serde_json::to_string(&req).expect("Serialization failed");
    //     assert!(serialized.contains("Testing serialization"));
    // }
}
