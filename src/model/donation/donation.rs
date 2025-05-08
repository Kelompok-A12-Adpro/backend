use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Donation {
    pub id: i32,
    pub user_id: i32,
    pub campaign_id: i32,
    pub amount: f64,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewDonationRequest {
   pub campaign_id: i32,
   pub amount: f64,
   pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDonationMessageRequest {
    pub message: Option<String>,
}
