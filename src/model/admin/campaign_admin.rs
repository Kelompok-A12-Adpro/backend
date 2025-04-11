use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CampaignStatus {
    PendingVerification,
    Rejected,
    Active,
    Completed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminCampaignSummary {
    pub id: i32,
    pub title: String,
    pub fundraiser_id: i32,
    pub fundraiser_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub target_amount: f64,
    pub collected_amount: f64,
    pub status: CampaignStatus,
}

#[derive(Debug, Deserialize)]
pub struct CampaignVerificationRequest {
    pub approved: bool,
    pub reason: Option<String>,
}
