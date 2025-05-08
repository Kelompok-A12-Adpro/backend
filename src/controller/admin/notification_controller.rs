use crate::errors::AppError;
use crate::model::admin::notification::{CreateNotificationRequest, Notification};
use rocket::serde::json::Json;
use rocket::{delete, get, post, routes};
use std::sync::atomic::{AtomicI32, Ordering};

// Static counter for generating notification IDs
static NEXT_ID: AtomicI32 = AtomicI32::new(1);

// Placeholder for simplified controllers
#[get("/notifications")]
fn get_notifications() -> Json<Vec<Notification>> {
    Json(Vec::new())
}

#[post("/notifications", format = "json", data = "<notification_req>")]
fn create_notification(
    notification_req: Json<CreateNotificationRequest>,
) -> Result<Json<Notification>, AppError> {
    // Simple validation
    if notification_req.title.is_empty() || notification_req.content.is_empty() {
        return Err(AppError::ValidationError(
            "Title and content cannot be empty".to_string(),
        ));
    }

    let notification = Notification {
        id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        title: notification_req.title.clone(),
        content: notification_req.content.clone(),
        created_at: chrono::Utc::now(),
        target_type: notification_req.target_type.clone(),
        target_id: notification_req.target_id,
    };

    Ok(Json(notification))
}

#[delete("/notifications/<_notification_id>")]
fn delete_notification(_notification_id: i32) -> Result<(), AppError> {
    if _notification_id < 1 {
        return Err(AppError::NotFound("Notification not found".to_string()));
    }

    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![get_notifications, create_notification, delete_notification]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::admin::notification::NotificationTargetType;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;

    fn rocket() -> rocket::Rocket<rocket::Build> {
        rocket::build().mount("/admin", routes())
    }

    #[test]
    fn test_get_notifications_empty() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/admin/notifications").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        // In the placeholder, it returns an empty vec
        assert_eq!(response.into_string().unwrap(), "[]");
    }

    #[test]
    fn test_create_notification_success() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let notification_data = CreateNotificationRequest {
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };
        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&notification_data).unwrap())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.into_string().unwrap();
        let created_notification: Notification =
            serde_json::from_str(&body).expect("valid notification json");
        assert_eq!(created_notification.title, "Test Title");
        assert_eq!(created_notification.content, "Test Content");
        assert_eq!(created_notification.id, 1);
    }

    #[test]
    fn test_create_notification_validation_error() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let notification_data = CreateNotificationRequest {
            title: "".to_string(), // Empty title to trigger validation error
            content: "Test Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };
        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&notification_data).unwrap())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        assert!(
            response
                .into_string()
                .unwrap()
                .contains("Title and content cannot be empty")
        );
    }

    #[test]
    fn test_delete_notification_success() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        // Assuming notification with ID 1 exists or the endpoint doesn't check
        let response = client.delete("/admin/notifications/1").dispatch();

        // Placeholder returns Ok(()) which maps to Status::Ok with no body
        assert_eq!(response.status(), Status::Ok);
        assert!(response.into_string().unwrap_or_default().is_empty());
    }

    #[test]
    fn test_delete_notification_not_found() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.delete("/admin/notifications/-1").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }
}
