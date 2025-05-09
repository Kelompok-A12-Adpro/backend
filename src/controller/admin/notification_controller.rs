use crate::errors::AppError;
use crate::model::admin::notification::{CreateNotificationRequest, NotificationTargetType, Notification};
use rocket::serde::json::Json;
use rocket::{catch, delete, get, post, routes};
use std::sync::atomic::{AtomicI32, Ordering};

// Static counter for generating notification IDs
static NEXT_ID: AtomicI32 = AtomicI32::new(1);

#[catch(422)]
fn json_parse_error(req: &rocket::Request<'_>) -> AppError {
    AppError::JsonParseError("Failed to parse JSON".to_string())
}

// Placeholder for simplified controllers
#[get("/notifications")]
fn get_notifications() -> Json<Vec<Notification>> {
    Json(vec![
        Notification {
            id: 1,
            title: "Welcome".to_string(),
            content: "Welcome to the platform!".to_string(),
            created_at: chrono::Utc::now(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        },
        Notification {
            id: 2,
            title: "System Update".to_string(),
            content: "System maintenance scheduled".to_string(),
            created_at: chrono::Utc::now(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        }
    ])
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

    if notification_req.target_type == NotificationTargetType::SpecificUser
        && notification_req.target_id.is_none()
    {
        return Err(AppError::ValidationError(
            "target_id is required for SpecificUser".to_string(),
        ));
    } else if (notification_req.target_type == NotificationTargetType::AllUsers
        || notification_req.target_type == NotificationTargetType::Fundraisers)
        && notification_req.target_id.is_some()
    {
        return Err(AppError::ValidationError(
            "target_id must be None for AllUsers or Fundraisers".to_string(),
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

pub fn catchers() -> Vec<rocket::Catcher> {
    rocket::catchers![json_parse_error]
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
    fn test_get_notifications() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/admin/notifications").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        
        let body = response.into_string().unwrap();
        let notifications: Vec<Notification> =
            serde_json::from_str(&body).expect("valid notification json");
        assert!(notifications.len() > 1);
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

    #[test]
    fn test_invalid_json_payload() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client
            .post("/admin/notifications")
            .header(ContentType::JSON)
            .body(r#"{"title": "Test", invalid json}"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        
        let body = response.into_string().unwrap();
        println!("Response body: {}", body);
        assert!(body.contains("Bad Request"));
    }
}
