use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationTargetType {
    AllUsers,
    Fundraisers,
    SpecificUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminNotificationView {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub target_type: NotificationTargetType,
    pub target_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub content: String,
    pub target_type: NotificationTargetType,
    pub target_id: Option<i32>,
}
