use crate::errors::AppError;
use chrono::{DateTime, Utc};
use crate::repository::admin::statistics_repo::StatisticsRepository;
use std::sync::Arc;


#[derive(Debug, Clone, PartialEq)]
pub enum StatisticsPeriod {
    Daily,
    Weekly,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionStatistics {
    pub count: i32,
    pub total_amount: f64,
    pub period: StatisticsPeriod,
}

// Represents a user summary for the dashboard
#[derive(Debug, Clone, PartialEq)]
pub struct UserSummary {
    pub id: i32,
    pub username: String,
    pub registration_date: DateTime<Utc>,
    pub user_type: String, // "Fundraiser" or "Donatur"
}

pub struct PlatformStatisticsService {
    stats_repo: Arc<dyn StatisticsRepository>,
}

impl PlatformStatisticsService {
    pub fn new(stats_repo: Arc<dyn StatisticsRepository>) -> Self {
        PlatformStatisticsService { stats_repo }
    }

    pub async fn get_active_campaigns_count(&self) -> Result<i32, AppError> {
        unimplemented!()
    }

    pub async fn get_total_donations_amount(&self) -> Result<f64, AppError> {
        unimplemented!()
    }

    pub async fn get_registered_users_count(&self) -> Result<i32, AppError> {
        unimplemented!()
    }

    pub async fn get_recent_users(&self, limit: i32) -> Result<Vec<UserSummary>, AppError> {
        unimplemented!()
    }

    pub async fn get_transaction_statistics(&self, period: StatisticsPeriod) -> Result<TransactionStatistics, AppError> {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::admin::statistics::RecentUser as RepoRecentUser; // Alias repo model
    use crate::repository::admin::statistics_repo::StatisticsRepository;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use tokio;

    // Mock StatisticsRepository for testing
    struct MockStatsRepo {
        active_campaigns: i32,
        total_donations: f64,
        registered_users: i32,
        recent_users: Vec<RepoRecentUser>,
        daily_transactions: i32,
        weekly_transactions: i32,
    }

    impl MockStatsRepo {
         // Helper to create a default mock instance
        fn default() -> Self {
            MockStatsRepo {
                active_campaigns: 10,
                total_donations: 5000.0,
                registered_users: 100,
                recent_users: vec![
                    RepoRecentUser { id: 1, username: "user1".to_string(), registration_date: Utc::now(), user_type: "Donor".to_string() },
                    RepoRecentUser { id: 2, username: "user2".to_string(), registration_date: Utc::now(), user_type: "Fundraiser".to_string() },
                ],
                daily_transactions: 20,
                weekly_transactions: 150,
            }
        }
    }

    #[async_trait]
    impl StatisticsRepository for MockStatsRepo {
        async fn get_active_campaigns_count(&self) -> Result<i32, AppError> {
            Ok(self.active_campaigns)
        }
        async fn get_total_donations_amount(&self) -> Result<f64, AppError> {
            Ok(self.total_donations)
        }
        async fn get_registered_users_count(&self) -> Result<i32, AppError> {
            Ok(self.registered_users)
        }
        async fn get_recent_users(&self, limit: i32) -> Result<Vec<RepoRecentUser>, AppError> {
            let count = limit.min(self.recent_users.len() as i32) as usize;
            Ok(self.recent_users.iter().take(count).cloned().collect())
        }
        async fn get_daily_transaction_count(&self) -> Result<i32, AppError> {
            Ok(self.daily_transactions)
        }
        async fn get_weekly_transaction_count(&self) -> Result<i32, AppError> {
            Ok(self.weekly_transactions)
        }
    }

    fn setup() -> (PlatformStatisticsService, Arc<MockStatsRepo>) {
        let mock_repo = Arc::new(MockStatsRepo::default());
        let service = PlatformStatisticsService::new(mock_repo.clone());
        (service, mock_repo)
    }

    #[tokio::test]
    async fn test_get_active_campaigns_count() {
        let (service, mock_repo) = setup();
        let expected_count = mock_repo.active_campaigns;
        let count = service.get_active_campaigns_count().await.unwrap();
        assert_eq!(count, expected_count);
    }

    #[tokio::test]
    async fn test_get_total_donations_amount() {
        let (service, mock_repo) = setup();
        let expected_amount = mock_repo.total_donations;
        let amount = service.get_total_donations_amount().await.unwrap();
        assert_eq!(amount, expected_amount);
    }

    #[tokio::test]
    async fn test_get_registered_users_count() {
        let (service, mock_repo) = setup();
        let expected_count = mock_repo.registered_users;
        let count = service.get_registered_users_count().await.unwrap();
        assert_eq!(count, expected_count);
    }

    #[tokio::test]
    async fn test_get_recent_users_with_limit() {
        let (service, mock_repo) = setup();
        let limit = 1;
        let users = service.get_recent_users(limit).await.unwrap();
        assert_eq!(users.len(), limit as usize);
        assert_eq!(users[0].id, mock_repo.recent_users[0].id); // Check mapping
        assert_eq!(users[0].username, mock_repo.recent_users[0].username);
    }

    #[tokio::test]
    async fn test_get_recent_users_limit_exceeds() {
        let (service, mock_repo) = setup();
        let limit = 5; // More than available in mock
        let users = service.get_recent_users(limit).await.unwrap();
        assert_eq!(users.len(), mock_repo.recent_users.len());
    }

    #[tokio::test]
    async fn test_get_transaction_statistics_daily() {
        let (service, mock_repo) = setup();
        let stats = service.get_transaction_statistics(StatisticsPeriod::Daily).await.unwrap();
        assert_eq!(stats.period, StatisticsPeriod::Daily);
        assert_eq!(stats.count, mock_repo.daily_transactions);
        // TODO: Update assertion for total_amount when implemented properly
        assert_eq!(stats.total_amount, 100.0); // Matches current placeholder
    }

    #[tokio::test]
    async fn test_get_transaction_statistics_weekly() {
        let (service, mock_repo) = setup();
        let stats = service.get_transaction_statistics(StatisticsPeriod::Weekly).await.unwrap();
        assert_eq!(stats.period, StatisticsPeriod::Weekly);
        assert_eq!(stats.count, mock_repo.weekly_transactions);
         // TODO: Update assertion for total_amount when implemented properly
        assert_eq!(stats.total_amount, 700.0); // Matches current placeholder
    }
}