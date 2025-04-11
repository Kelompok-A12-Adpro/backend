use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Campaign {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: String,
    pub target_amount: f64,
    pub collected_amount: f64,
    pub status: CampaignStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub evidence_url: Option<String>,
    pub evidence_uploaded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "campaign_status", rename_all = "snake_case")]
pub enum CampaignStatus {
    PendingVerification,
    Active,
    Rejected,
    Completed,
    Closed,
}

#[derive(Debug, Deserialize)]
pub struct NewCampaignRequest {
    pub name: String,
    pub description: String,
    pub target_amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct EvidenceUploadRequest {
    pub evidence_url: String,
}