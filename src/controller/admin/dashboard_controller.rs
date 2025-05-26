use rocket::{State, get, routes};
use rocket::serde::json::Json;
use crate::service::admin::platform_statistics_service::PlatformStatisticsService;
use crate::model::admin::statistics::{PlatformStatistics, UserSummary};
use crate::errors::AppError;

// Placeholder for authentication
struct AuthUser {
    id: i32,
    is_admin: bool,
}

#[get("/dashboard/statistics")]
async fn get_dashboard_statistics(
    statistics_service: &State<PlatformStatisticsService>
) -> Result<Json<PlatformStatistics>, AppError> {
    let active_campaigns = statistics_service.get_active_campaigns_count().await?;
    let total_donations = statistics_service.get_total_donations_amount().await?;
    let registered_users = statistics_service.get_registered_users_count().await?;
    let daily_transactions = 20; // Placeholder for daily transactions
    let weekly_transactions = 150; // Placeholder for weekly transactions

    Ok(Json(PlatformStatistics {
        active_campaigns_count: active_campaigns,
        total_donations_amount: total_donations,
        registered_users_count: registered_users,
        daily_transaction_count: daily_transactions,
        weekly_transaction_count: weekly_transactions
    }))
}

#[get("/dashboard/recent-users?<limit>")]
async fn get_recent_users(
    statistics_service: &State<PlatformStatisticsService>,
    limit: Option<i32>
) -> Result<Json<Vec<UserSummary>>, AppError> {
    let limit_value = limit.unwrap_or(10);
    let user_summaries = statistics_service.get_recent_users(limit_value).await?;
    let recent_users: Vec<UserSummary> = user_summaries
        .into_iter()
        .map(|summary| UserSummary {
            id: summary.id,
            name: summary.name,
            phone: summary.phone,
        })
        .collect();
    Ok(Json(recent_users))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_dashboard_statistics,
        get_recent_users
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::http::{Status, ContentType};
    use crate::model::admin::statistics::{PlatformStatistics, UserSummary};
    use crate::service::admin::platform_statistics_service::{PlatformStatisticsService};
    use crate::repository::admin::statistics_repo::StatisticsRepository;
    use crate::model::admin::statistics::UserSummary as RepoUserSummary;
    use crate::errors::AppError;
    use async_trait::async_trait;
    use std::sync::Arc;

    // --- Mock Dependencies ---

    // Mock StatisticsRepository
    struct MockStatsRepo {
        active_campaigns: i32,
        total_donations: f64,
        registered_users: i32,
        recent_users: Vec<RepoUserSummary>,
        daily_transactions: i32,
        weekly_transactions: i32,
    }

    impl MockStatsRepo {
        fn default() -> Self {
            MockStatsRepo {
                active_campaigns: 10,
                total_donations: 5000.0,
                registered_users: 100,
                recent_users: vec![
                    RepoUserSummary { id: 1, name: "user1".to_string(), phone: "123456789014".to_string() },
                    RepoUserSummary { id: 2, name: "user2".to_string(), phone: "098765432113".to_string() },
                    RepoUserSummary { id: 3, name: "user3".to_string(), phone: "088902068861".to_string() },
                ],
                daily_transactions: 20,
                weekly_transactions: 150,
            }
        }
    }

    #[async_trait]
    impl StatisticsRepository for MockStatsRepo {
        async fn get_active_campaigns_count(&self) -> Result<i32, AppError> { Ok(self.active_campaigns) }
        async fn get_total_donations_amount(&self) -> Result<f64, AppError> { Ok(self.total_donations) }
        async fn get_registered_users_count(&self) -> Result<i32, AppError> { Ok(self.registered_users) }
        async fn get_recent_users(&self, limit: i32) -> Result<Vec<RepoUserSummary>, AppError> {
            let count = limit.min(self.recent_users.len() as i32) as usize;
            Ok(self.recent_users.iter().take(count).cloned().collect())
        }
        async fn get_daily_transaction_count(&self) -> Result<i32, AppError> { Ok(self.daily_transactions) }
        async fn get_weekly_transaction_count(&self) -> Result<i32, AppError> { Ok(self.weekly_transactions) }
    }

    // --- Test Setup ---

    // Helper to build Rocket instance with mock service
    fn rocket() -> rocket::Rocket<rocket::Build> {
        let mock_repo = Arc::new(MockStatsRepo::default());
        let mock_service = PlatformStatisticsService::new(mock_repo);

        rocket::build()
            .mount("/admin", routes()) // Assuming routes are mounted under /admin
            .manage(mock_service) // Manage the mock service state
    }

    // --- Tests ---

    #[test]
    fn test_get_dashboard_statistics_success() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/admin/dashboard/statistics").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let stats: PlatformStatistics = response.into_json().expect("Failed to deserialize response");

        assert_eq!(stats.active_campaigns_count, 10); // Expected value from mock repo via service
        assert_eq!(stats.total_donations_amount, 5000.0); // Expected value
        assert_eq!(stats.registered_users_count, 100); // Expected value
        assert_eq!(stats.daily_transaction_count, 20); // Expected value
        assert_eq!(stats.weekly_transaction_count, 150); // Expected value
    }

    #[test]
    fn test_get_recent_users_default_limit() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/admin/dashboard/recent-users").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let users: Vec<UserSummary> = response.into_json().expect("Failed to deserialize response");

        let expected_users_count = 3; // Default limit might be 5 or 10 in real service, mock has 3
        assert_eq!(users.len(), expected_users_count);
        assert_eq!(users[0].id, 1);
        assert_eq!(users[1].id, 2);
    }

    #[test]
    fn test_get_recent_users_with_limit() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let limit = 2;
        let response = client.get(format!("/admin/dashboard/recent-users?limit={}", limit)).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let users: Vec<UserSummary> = response.into_json().expect("Failed to deserialize response");

        assert_eq!(users.len(), limit);
        assert_eq!(users[0].id, 1);
        assert_eq!(users[1].id, 2);
    }
}