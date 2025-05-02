use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Blocked,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserView {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub status: UserStatus,
    pub user_type: String, // "Fundraiser", "Donor", etc.
}

#[derive(Debug, Deserialize)]
pub struct UserActionRequest {
    pub reason: String,
}
