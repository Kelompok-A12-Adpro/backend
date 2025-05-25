use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCampaignSubscription {
    pub user_email: String,
    pub start_at: DateTime<Utc>,
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
}
