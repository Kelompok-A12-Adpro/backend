use rocket::{State, get, routes};
use rocket::serde::json::Json;
use crate::service::admin::platform_statistics_service::PlatformStatisticsService;
use crate::model::admin::statistics::{PlatformStatistics, RecentUser};
use crate::errors::AppError;

// Placeholder for authentication
struct AuthUser {
    id: i32,
    is_admin: bool,
}

#[get("/admin/dashboard/statistics")]
async fn get_dashboard_statistics(
    statistics_service: &State<PlatformStatisticsService>
) -> Result<Json<PlatformStatistics>, AppError> {
    // Admin auth check would go below here
    
    // For now, return placeholder
    Ok(Json(PlatformStatistics {
        active_campaigns_count: 0,
        total_donations_amount: 0.0,
        registered_users_count: 0,
        daily_transaction_count: 0,
        weekly_transaction_count: 0
    }))
}

#[get("/admin/dashboard/recent-users?<limit>")]
async fn get_recent_users(
    statistics_service: &State<PlatformStatisticsService>,
    limit: Option<i32>
) -> Result<Json<Vec<RecentUser>>, AppError> {
    // Admin auth check would go below here...
    // For now, return empty vector
    Ok(Json(Vec::new()))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_dashboard_statistics,
        get_recent_users
    ]
}
