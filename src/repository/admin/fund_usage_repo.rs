use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::admin::fund_usage::{AdminFundUsageView, FundUsageStatus};

#[async_trait]
pub trait FundUsageRepository: Send + Sync {
    async fn get_pending_verifications(&self) -> Result<Vec<AdminFundUsageView>, AppError>;
    async fn get_fund_usage_by_id(&self, usage_id: i32) -> Result<Option<AdminFundUsageView>, AppError>;
    async fn update_fund_usage_status(&self, usage_id: i32, status: FundUsageStatus, notes: Option<String>) -> Result<bool, AppError>;
}

// Implementation will be added later
