use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FundUsageStatus {
    PendingVerification,
    Approved,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminFundUsageView {
    pub id: i32,
    pub campaign_id: i32,
    pub campaign_title: String,
    pub amount: f64,
    pub description: String,
    pub proof_url: String,
    pub submitted_at: DateTime<Utc>,
    pub status: FundUsageStatus,
    pub admin_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FundUsageVerificationRequest {
    pub approve: bool,
    pub notes: Option<String>,
}
