use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::admin::statistics::{PlatformStatistics, RecentUser};

#[async_trait]
pub trait StatisticsRepository: Send + Sync {
    async fn get_active_campaigns_count(&self) -> Result<i32, AppError>;
    async fn get_total_donations_amount(&self) -> Result<f64, AppError>;
    async fn get_registered_users_count(&self) -> Result<i32, AppError>;
    async fn get_recent_users(&self, limit: i32) -> Result<Vec<RecentUser>, AppError>;
    async fn get_daily_transaction_count(&self) -> Result<i32, AppError>;
    async fn get_weekly_transaction_count(&self) -> Result<i32, AppError>;
}

// Implementation will be added later
