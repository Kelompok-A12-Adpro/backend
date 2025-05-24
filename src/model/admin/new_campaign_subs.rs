use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCampaignSubscription {
    pub user_email: String,
    pub start_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCampaignSubscriptionRequest {
    pub user_email: String,
    pub start_at: DateTime<Utc>,
}

pub fn validate_request(req: &NewCampaignSubscriptionRequest) -> Result<(), String> {
    if req.user_email.is_empty() {
        return Err("User email cannot be empty".to_string());
    }
    if req.start_at.timestamp() <= 0 {
        return Err("Start time must be a valid timestamp".to_string());
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn get_now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn test_new_campaign_subscription_instantiation() {
        let now = get_now();
        let subscription = NewCampaignSubscription {
            user_email: "dummy@example.com".to_string(),
            start_at: now,
        };
        assert_eq!(subscription.user_email, "dummy@example.com");
        assert_eq!(subscription.start_at, now);
    }

    #[test]
    fn test_new_campaign_subscription_request_instantiation() {
        let now = get_now();
        let subscription_request = NewCampaignSubscriptionRequest {
            user_email: "dummy@example.com".to_string(),
            start_at: now,
        };
        assert_eq!(subscription_request.user_email, "dummy@example.com");
        assert_eq!(subscription_request.start_at, now);
    }

    #[test]
    fn test_validate_request_success() {
        let now = get_now();
        let subscription_request = NewCampaignSubscriptionRequest {
            user_email: "dummy@example.com".to_string(),
            start_at: now,
        };
        let result = validate_request(&subscription_request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_request_empty_email() {
        let now = get_now();
        let invalid_request = NewCampaignSubscriptionRequest {
            user_email: "".to_string(),
            start_at: now,
        };
        let result = validate_request(&invalid_request);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User email cannot be empty");
    }

    #[test]
    fn test_validate_request_invalid_start_time() {
        let invalid_request = NewCampaignSubscriptionRequest {
            user_email: "dummy@example.com".to_string(),
            start_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(), // Invalid timestamp
        };
        let result = validate_request(&invalid_request);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Start time must be a valid timestamp");
    }
}
