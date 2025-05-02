use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlatformStatistics {
    pub active_campaigns_count: i32,
    pub total_donations_amount: f64,
    pub registered_users_count: i32,
    pub daily_transaction_count: i32,
    pub weekly_transaction_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentUser {
    pub id: i32,
    pub username: String,
    pub registration_date: DateTime<Utc>,
    pub user_type: String, // "Fundraiser" or "Donor"
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json;

    #[test]
    fn test_platform_statistics_instantiation() {
        let stats = PlatformStatistics {
            active_campaigns_count: 5,
            total_donations_amount: 12345.67,
            registered_users_count: 150,
            daily_transaction_count: 25,
            weekly_transaction_count: 175,
        };

        assert_eq!(stats.active_campaigns_count, 5);
        assert_eq!(stats.total_donations_amount, 12345.67);
        assert_eq!(stats.registered_users_count, 150);
        assert_eq!(stats.daily_transaction_count, 25);
        assert_eq!(stats.weekly_transaction_count, 175);
    }

    #[test]
    fn test_platform_statistics_serialization_deserialization() {
        let stats = PlatformStatistics {
            active_campaigns_count: 10,
            total_donations_amount: 987.65,
            registered_users_count: 200,
            daily_transaction_count: 30,
            weekly_transaction_count: 210,
        };

        let serialized = serde_json::to_string(&stats).expect("Serialization failed");
        let deserialized: PlatformStatistics = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(stats.active_campaigns_count, deserialized.active_campaigns_count);
        assert_eq!(stats.total_donations_amount, deserialized.total_donations_amount);
        assert_eq!(stats.registered_users_count, deserialized.registered_users_count);
        assert_eq!(stats.daily_transaction_count, deserialized.daily_transaction_count);
        assert_eq!(stats.weekly_transaction_count, deserialized.weekly_transaction_count);
    }

     #[test]
    fn test_platform_statistics_clone() {
        let stats = PlatformStatistics {
            active_campaigns_count: 15,
            total_donations_amount: 111.22,
            registered_users_count: 250,
            daily_transaction_count: 35,
            weekly_transaction_count: 245,
        };
        let cloned_stats = stats.clone();

        // Assert fields are equal (basic check for Clone correctness)
        assert_eq!(stats.active_campaigns_count, cloned_stats.active_campaigns_count);
        assert_eq!(stats.total_donations_amount, cloned_stats.total_donations_amount);
        assert_eq!(stats.registered_users_count, cloned_stats.registered_users_count);
        assert_eq!(stats.daily_transaction_count, cloned_stats.daily_transaction_count);
        assert_eq!(stats.weekly_transaction_count, cloned_stats.weekly_transaction_count);

        // Optionally, assert they are not the same memory location if needed,
        // though derive(Clone) typically ensures a new allocation for owned types like String.
        // For simple types like i32/f64, the values are copied.
    }


    #[test]
    fn test_recent_user_instantiation() {
        let now = Utc::now();
        let user = RecentUser {
            id: 101,
            username: "testuser".to_string(),
            registration_date: now,
            user_type: "Donor".to_string(),
        };

        assert_eq!(user.id, 101);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.registration_date, now);
        assert_eq!(user.user_type, "Donor");
    }

    #[test]
    fn test_recent_user_serialization_deserialization() {
        let now = Utc::now();
        let user = RecentUser {
            id: 202,
            username: "anotheruser".to_string(),
            registration_date: now,
            user_type: "Fundraiser".to_string(),
        };

        let serialized = serde_json::to_string(&user).expect("Serialization failed");
        let deserialized: RecentUser = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.username, deserialized.username);
        assert_eq!(user.registration_date, deserialized.registration_date);
        assert_eq!(user.user_type, deserialized.user_type);
    }

    #[test]
    fn test_recent_user_clone() {
        let now = Utc::now();
        let user = RecentUser {
            id: 303,
            username: "cloneuser".to_string(),
            registration_date: now,
            user_type: "Donor".to_string(),
        };
        let cloned_user = user.clone();

        // Assert fields are equal
        assert_eq!(user.id, cloned_user.id);
        assert_eq!(user.username, cloned_user.username);
        assert_eq!(user.registration_date, cloned_user.registration_date); // DateTime<Utc> implements Copy
        assert_eq!(user.user_type, cloned_user.user_type);

        // Assert String fields point to different memory locations after clone
        assert_ne!(user.username.as_ptr(), cloned_user.username.as_ptr());
        assert_ne!(user.user_type.as_ptr(), cloned_user.user_type.as_ptr());
    }
}
