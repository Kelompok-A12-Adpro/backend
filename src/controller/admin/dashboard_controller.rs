use std::sync::Arc;

use rocket::{State, get, routes};
use rocket::serde::json::Json;
use crate::service::admin::platform_statistics_service::{StatisticService};
use crate::model::admin::statistic::{CampaignStat, DataStatistic, DonationStat, RecentDonation, TransactionData};
use crate::errors::AppError;
use crate::controller::auth::auth::AuthUser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

fn admin_check(auth_user: AuthUser) -> Result<(), AppError> {
    if !auth_user.is_admin {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}

#[get("/statistics")]
async fn get_statistics(
    auth_user: AuthUser,
    statistic_service: &State<Arc<StatisticService>>
) -> Result<Json<ApiResponse<DataStatistic>>, AppError> {
    admin_check(auth_user)?;

    let data_stats = statistic_service.get_data_statistic_count().await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Dashboard statistics retrieved successfully".to_string(),
        data: Some(data_stats),
    }))
}

#[get("/statistics/daily-transactions")]
async fn get_daily_transaction_statistics(
    auth_user: AuthUser,
    service: &State<Arc<StatisticService>>,
) -> Result<Json<ApiResponse<Vec<TransactionData>>>, AppError> {
    admin_check(auth_user)?;

    let transactions = service.get_daily_transaction_statistics().await?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Daily transaction statistics retrieved successfully".to_string(),
        data: Some(transactions),
    }))
}

#[get("/statistics/weekly-transactions")]
async fn get_weekly_transaction_statistics(
    auth_user: AuthUser,
    service: &State<Arc<StatisticService>>,
) -> Result<Json<ApiResponse<Vec<TransactionData>>>, AppError> {
    admin_check(auth_user)?;

    let transactions = service.get_weekly_transaction_statistics().await?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Weekly transaction statistics retrieved successfully".to_string(),
        data: Some(transactions),
    }))
}

#[get("/statistics/recent-transactions?<limit>")]
async fn get_recent_transactions(
    auth_user: AuthUser,
    service: &State<Arc<StatisticService>>,
    limit: Option<i64>,
) -> Result<Json<ApiResponse<Vec<RecentDonation>>>, AppError> {
    admin_check(auth_user)?;

    let transactions = service.get_recent_transactions(limit).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Recent transactions retrieved successfully".to_string(),
        data: Some(transactions),
    }))
}

#[get("/campaigns/statistics")]
async fn get_campaign_statistics(
    auth_user: AuthUser,
    service: &State<Arc<StatisticService>>,
) -> Result<Json<ApiResponse<Vec<CampaignStat>>>, AppError> {
    admin_check(auth_user)?;

    let campaign_stats = service.get_all_campaigns_with_progress().await?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Campaign statistics retrieved successfully".to_string(),
        data: Some(campaign_stats),
    }))
}

#[get("/donations/statistics")]
async fn get_donation_statistics(
    auth_user: AuthUser,
    service: &State<Arc<StatisticService>>,
) -> Result<Json<ApiResponse<Vec<DonationStat>>>, AppError> {
    admin_check(auth_user)?;

    let donation_stats = service.get_all_donations().await?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Donation statistics retrieved successfully".to_string(),
        data: Some(donation_stats),
    }))
}


pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_statistics,
        get_daily_transaction_statistics,
        get_weekly_transaction_statistics,
        get_recent_transactions,
        get_campaign_statistics,
        get_donation_statistics
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