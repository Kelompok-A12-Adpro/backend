use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformStatistics {
    pub active_campaigns_count: i32,
    pub total_donations_amount: f64,
    pub registered_users_count: i32,
    pub daily_transaction_count: i32,
    pub weekly_transaction_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentUser {
    pub id: i32,
    pub username: String,
    pub registration_date: DateTime<Utc>,
    pub user_type: String, // "Fundraiser" or "Donor"
}
