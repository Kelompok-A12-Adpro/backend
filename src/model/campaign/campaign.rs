use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CampaignStatus {
    PendingVerification,
    Active,
    Rejected,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: String,
    pub target_amount: f64,
    pub collected_amount: f64,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub image_url: Option<String>,
    pub evidence_url: Option<String>,
    pub evidence_uploaded_at: Option<DateTime<Utc>>,
    pub status: CampaignStatus,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}