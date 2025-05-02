use crate::service::notification::notification_observer::NotificationObserver;
use crate::model::admin::notification::Notification;

pub struct UserNotificationObserver {
    observer_id: String,
    user_id: i32,
}

impl UserNotificationObserver {
    pub fn new(user_id: i32) -> Self {
        UserNotificationObserver {
            observer_id: format!("user_{}", user_id),
            user_id,
        }
    }
}

impl NotificationObserver for UserNotificationObserver {
    fn update(&self, notification: &Notification) {
        // Logic to send notification to user
        println!("User {} received notification: {}", self.user_id, notification.title);
    }
    
    fn id(&self) -> &str {
        &self.observer_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::admin::notification::{Notification, NotificationTargetType};
    use chrono::Utc;
    use std::sync::Arc;

    #[test]
    fn test_new_and_id() {
        let user_id = 987;
        let observer = UserNotificationObserver::new(user_id);
        let expected_id = format!("user_{}", user_id);

        assert_eq!(observer.user_id, user_id);
        assert_eq!(observer.observer_id, expected_id);
        assert_eq!(observer.id(), expected_id);
    }

    #[test]
    fn test_update_call() {
        let user_id = 654;
        let observer = UserNotificationObserver::new(user_id);

        let notification = Notification {
            id: 20,
            title: "General Announcement".to_string(),
            content: "Platform maintenance scheduled.".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers, // Example
            target_id: None, // Example
        };

        // This test primarily ensures the update function can be called
        // without panicking with the current placeholder implementation.
        observer.update(&notification);
        // No panic means the basic call worked.
    }

    // Test Send + Sync bounds (compilation check)
    #[test]
    fn test_thread_safety() {
        let observer = Arc::new(UserNotificationObserver::new(321));
        let observer_clone = observer.clone();

        let notification = Notification {
            id: 21,
            title: "User Thread Safety Test".to_string(),
            content: "Testing user observer from another thread.".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::SpecificUser, // Example
            target_id: Some(321), // Example
        };

        let handle = std::thread::spawn(move || {
            // Call update from another thread
            observer_clone.update(&notification);
            // Return the ID from the thread
            observer_clone.id().to_string()
        });

        let returned_id = handle.join().unwrap();

        // Check if the ID is correct from the main thread and the spawned thread
        assert_eq!(observer.id(), "user_321");
        assert_eq!(returned_id, "user_321");
    }
}