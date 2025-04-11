use crate::errors::AppError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum FundUsageStatus {
    PendingVerification,
    Approved,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct FundUsage {
    pub id: i32,
    pub campaign_id: i32,
    pub amount: f64,
    pub description: String,
    pub proof_url: String,
    pub submitted_at: DateTime<Utc>,
    pub status: FundUsageStatus,
    pub admin_notes: Option<String>,
}

pub struct FundUsageVerificationService {}

impl FundUsageVerificationService {
    pub fn new() -> Self {
        FundUsageVerificationService {}
    }
    
    pub async fn get_pending_verifications(&self) -> Result<Vec<FundUsage>, AppError> {
        // Will fetch all pending fund usage verifications
        unimplemented!()
    }
    
    pub async fn get_fund_usage_by_id(&self, usage_id: i32) -> Result<Option<FundUsage>, AppError> {
        // Will fetch a specific fund usage record
        unimplemented!()
    }
    
    pub async fn verify_fund_usage(&self, usage_id: i32, approve: bool, notes: Option<String>) -> Result<FundUsage, AppError> {
        // Will approve or reject a fund usage verification
        unimplemented!()
    }
}
