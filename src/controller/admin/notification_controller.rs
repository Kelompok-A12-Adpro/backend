use rocket::{get, post, delete, routes};
use rocket::serde::json::Json;
use crate::model::admin::notification::{Notification, CreateNotificationRequest};
use crate::errors::AppError;

// Placeholder for simplified controllers
#[get("/notifications")]
fn get_notifications() -> Json<Vec<Notification>> {
    Json(Vec::new())
}

#[post("/notifications", format = "json", data = "<notification_req>")]
fn create_notification(notification_req: Json<CreateNotificationRequest>) -> Result<Json<Notification>, AppError> {
    // Simple validation
    if notification_req.title.is_empty() || notification_req.content.is_empty() {
        return Err(AppError::ValidationError("Title and content cannot be empty".to_string()));
    }
    
    Ok(Json(Notification {
        id: 1,
        title: notification_req.title.clone(),
        content: notification_req.content.clone(),
        created_at: chrono::Utc::now(),
        target_type: notification_req.target_type.clone(),
        target_id: notification_req.target_id,
    }))
}

#[delete("/notifications/<_notification_id>")]
fn delete_notification(_notification_id: i32) -> Result<(), AppError> {
    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_notifications,
        create_notification,
        delete_notification
    ]
}
