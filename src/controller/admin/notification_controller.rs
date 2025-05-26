use std::sync::Arc;

use crate::controller::auth::auth::AuthUser;
use crate::errors::AppError;
use crate::model::admin::notification::{
    CreateNotificationRequest, Notification,
};
use crate::service::admin::notification::notification_service::NotificationService;
use autometrics::autometrics;
use rocket::serde::json::Json;
use rocket::{catch, delete, get, post, routes, State};
use serde::{Deserialize, Serialize};

#[catch(422)]
fn json_parse_error(req: &rocket::Request<'_>) -> AppError {
    AppError::JsonParseError(format!(
        "Invalid JSON payload received at {} {}. Please check your JSON syntax and structure.",
        req.method(),
        req.uri().path()
    ))
}

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


//////////////////////
///// Admin Side /////
//////////////////////

#[get("/notifications")]
async fn get_notifications_admin(
    auth_user: AuthUser,
    service: &State<Arc<NotificationService>>,
) -> Result<Json<ApiResponse<Vec<Notification>>>, AppError> {
    admin_check(auth_user)?;

    let notifications = service.get_all_notifications().await.unwrap_or_default();
    Ok(Json(ApiResponse {
        success: true,
        message: "Notifications retrieved successfully".to_string(),
        data: Some(notifications),
    }))
}

#[post("/notifications", format = "json", data = "<notification_data>")]
async fn create_notification(
    auth_user: AuthUser,
    notification_data: Json<CreateNotificationRequest>,
    service: &State<Arc<NotificationService>>,
) -> Result<Json<ApiResponse<Notification>>, AppError> {
    admin_check(auth_user)?;

    let request = notification_data.into_inner();

    if request.title.is_empty() || request.content.is_empty() {
        return Err(AppError::InvalidOperation(
            "Title and content cannot be empty".to_string(),
        ));
    }

    let notification = service
        .notify(request)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Notification created successfully".to_string(),
        data: Some(notification),
    }))
}

#[delete("/notifications/<notification_id>")]
async fn delete_notification(
    auth_user: AuthUser,
    service: &State<Arc<NotificationService>>,
    notification_id: i32,
) -> Result<Json<ApiResponse<String>>, AppError> {
    admin_check(auth_user)?;

    service
        .delete_notification(notification_id)
        .await
        .map_err(|_| AppError::NotFound(format!(
            "Notification with ID {} not found",
            notification_id
        )))?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Notification deleted successfully".to_string(),
        data: None,
    }))
}


/////////////////////
///// User Side /////
/////////////////////

#[get("/notifications")]
#[autometrics]
async fn get_notifications(
    auth_user: AuthUser,
    service: &State<Arc<NotificationService>>,
) -> Json<ApiResponse<Vec<Notification>>> {
    let notifications = service
        .get_notifications_for_user(auth_user.email)
        .await
        .unwrap_or_default();

    Json(ApiResponse {
        success: true,
        message: "Notifications retrieved successfully".to_string(),
        data: Some(notifications),
    })
}

#[post("/subscribe", format = "json")]
#[autometrics]
async fn subscribe_to_notification(
    auth_user: AuthUser,
    service: &State<Arc<NotificationService>>,
) -> Json<ApiResponse<String>> {
    let user_email = auth_user.email.clone();

    match service.subscribe(user_email).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Subscribed successfully".to_string(),
            data: None,
        }),
        Err(_) => Json(ApiResponse {
            success: false,
            message: "Failed to subscribe".to_string(),
            data: None,
        }),
    }
}

#[post("/unsubscribe", format = "json")]
#[autometrics]
async fn unsubscribe_from_notification(
    auth_user: AuthUser,
    service: &State<Arc<NotificationService>>,
) -> Json<ApiResponse<String>> {
    let user_email = auth_user.email.clone();

    match service.unsubscribe(user_email).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Unsubscribed successfully".to_string(),
            data: None,
        }),
        Err(_) => Json(ApiResponse {
            success: false,
            message: "Failed to unsubscribe".to_string(),
            data: None,
        }),
    }
}

#[delete("/notifications/<notification_id>")]
#[autometrics]
async fn delete_notification_user(
    auth_user: AuthUser,
    service: &State<Arc<NotificationService>>,
    notification_id: i32,
) -> Json<ApiResponse<String>> {
    match service
        .delete_notification_for_user(notification_id, auth_user.email)
        .await
    {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Notification deleted successfully".to_string(),
            data: None,
        }),
        Err(_) => Json(ApiResponse {
            success: false,
            message: format!("Notification with ID {} not found", notification_id),
            data: None,
        }),
    }
}

pub fn user_routes() -> Vec<rocket::Route> {
    routes![
        get_notifications,
        subscribe_to_notification,
        unsubscribe_from_notification,
        delete_notification_user
    ]
}

pub fn admin_routes() -> Vec<rocket::Route> {
    routes![
        get_notifications_admin,
        create_notification,
        delete_notification
    ]
}

pub fn catchers() -> Vec<rocket::Catcher> {
    rocket::catchers![json_parse_error]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::db::get_test_pool;
    use crate::model::admin::notification::NotificationTargetType;
    use crate::repository::admin::new_campaign_subs_repo::DbNewCampaignSubscriptionRepository;
    use crate::repository::admin::notification_repo::DbNotificationRepository;
    use crate::service::admin::notification::notification_observer::SubscriberService;
    use rocket::http::{ContentType, Status};
    use rocket::local::asynchronous::Client;

    async fn rocket() -> rocket::Rocket<rocket::Build> {
        let pool = get_test_pool().await;

        // Repos
        let notification_repo = Arc::new(DbNotificationRepository::new(pool.clone()));
        let new_campaign_subs_repo = Arc::new(DbNewCampaignSubscriptionRepository::new(pool.clone()));

        // Design Patterns
        let subscriber_service = Arc::new(SubscriberService::new(notification_repo.clone()));
        
        // Services
        let notification_service = NotificationService::new(
            notification_repo,
            new_campaign_subs_repo,
            subscriber_service.clone(),
        );

        rocket::build()
            .mount("/admin", admin_routes())
            .mount("/user", user_routes())
            .register("/", catchers())
            .manage(notification_service)
    }

    #[tokio::test]
    async fn test_get_notifications() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");

        let notification_data = CreateNotificationRequest {
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&notification_data).unwrap())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response = client.get("/admin/notifications").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let body = response.into_string().await.unwrap();
        let api_response: ApiResponse<Vec<Notification>> =
            serde_json::from_str(&body).expect("valid notification json");
        let notifications = api_response.data.unwrap_or_default();
        assert!(notifications.len() > 1);
    }

    #[tokio::test]
    async fn test_create_notification_success() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let notification_data = CreateNotificationRequest {
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&notification_data).unwrap())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        
        let body = response.into_string().await.unwrap();
        let api_response: ApiResponse<Notification> =
            serde_json::from_str(&body).expect("valid notification json");
        let created_notification = api_response.data.unwrap();
        
        assert_eq!(created_notification.title, "Test Title");
        assert_eq!(created_notification.content, "Test Content");
        assert!(created_notification.id > 0);
    }

    #[tokio::test]
    async fn test_create_notification_validation_error() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let notification_data = CreateNotificationRequest {
            title: "".to_string(), // Empty title to trigger validation error
            content: "Test Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&notification_data).unwrap())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::BadRequest);
        assert!(response
            .into_string()
            .await
            .unwrap()
            .contains("Title and content cannot be empty"));
    }

    #[tokio::test]
    async fn test_delete_notification_success() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        
        let notification_data = CreateNotificationRequest {
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&notification_data).unwrap())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response = client.delete("/admin/notifications/1").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        
        let body = response.into_string().await.unwrap();
        let api_response: ApiResponse<String> = serde_json::from_str(&body).expect("valid json response");
        assert_eq!(api_response.success, true);
        assert_eq!(api_response.message, "Notification deleted successfully");
    }

    #[tokio::test]
    async fn test_delete_notification_not_found() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client.delete("/admin/notifications/-1").dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[tokio::test]
    async fn test_invalid_json_payload() {
        let client = Client::tracked(rocket().await).await.expect("valid rocket instance");
        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(r#"{"title": "Test", invalid json}"#)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::BadRequest);

        let body = response.into_string().await.unwrap();
        println!("Response body: {}", body);
        assert!(body.contains("Bad Request"));
    }
}
