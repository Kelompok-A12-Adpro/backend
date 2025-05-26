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
    use rocket::local::asynchronous::Client;
    use rocket::http::{Status, ContentType};
    use crate::db::get_test_pool;
    use crate::service::admin::platform_statistics_service::StatisticService;

    // Helper to build Rocket instance with mock service
    async fn rocket() -> rocket::Rocket<rocket::Build> {
        let mock_service = StatisticService::new(get_test_pool().await);

        rocket::build()
            .mount("/admin", routes()) // Assuming routes are mounted under /admin
            .manage(mock_service)
    }

    #[tokio::test]
    async fn test_get_statistics_success() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.get("/admin/statistics").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let stats: ApiResponse<DataStatistic> = response.into_json().await.expect("Failed to deserialize response");

        let data_stats = stats.data.expect("Expected data to be present");
        
        assert!(stats.success);
        assert_eq!(stats.message, "Dashboard statistics retrieved successfully");
        assert!(data_stats.active_campaigns_count >= 0);
        assert!(data_stats.total_donations_amount >= 0);
        assert!(data_stats.daily_transaction_count >= 0);
        assert!(data_stats.weekly_transaction_count >= 0);
    }

    #[tokio::test]
    async fn test_get_daily_transaction_statistics_success() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.get("/admin/statistics/daily-transactions").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let result: ApiResponse<Vec<TransactionData>> = response.into_json().await.expect("Failed to deserialize response");
        
        assert!(result.success);
        assert_eq!(result.message, "Daily transaction statistics retrieved successfully");
        assert!(result.data.is_some());
    }

    #[tokio::test]
    async fn test_get_weekly_transaction_statistics_success() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.get("/admin/statistics/weekly-transactions").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let result: ApiResponse<Vec<TransactionData>> = response.into_json().await.expect("Failed to deserialize response");
        
        assert!(result.success);
        assert_eq!(result.message, "Weekly transaction statistics retrieved successfully");
        assert!(result.data.is_some());
    }

    #[tokio::test]
    async fn test_get_recent_transactions_success() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.get("/admin/statistics/recent-transactions").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let result: ApiResponse<Vec<RecentDonation>> = response.into_json().await.expect("Failed to deserialize response");
        
        assert!(result.success);
        assert_eq!(result.message, "Recent transactions retrieved successfully");
        assert!(result.data.is_some());
    }

    #[tokio::test]
    async fn test_get_recent_transactions_with_limit() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.get("/admin/statistics/recent-transactions?limit=5").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let result: ApiResponse<Vec<RecentDonation>> = response.into_json().await.expect("Failed to deserialize response");
        
        assert!(result.success);
        assert_eq!(result.message, "Recent transactions retrieved successfully");
        assert!(result.data.is_some());
    }
}