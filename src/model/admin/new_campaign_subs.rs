use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCampaignSubscription {
    pub user_id: i32,
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
            user_id: 1,
            start_at: now,
        };
        assert_eq!(subscription.user_id, 1);
        assert_eq!(subscription.start_at, now);
    }
}
