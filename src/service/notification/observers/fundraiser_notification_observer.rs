use crate::service::notification::notification_observer::NotificationObserver;
use crate::model::admin::notification::Notification;

pub struct FundraiserNotificationObserver {
    observer_id: String,
    fundraiser_id: i32,
}

impl FundraiserNotificationObserver {
    pub fn new(fundraiser_id: i32) -> Self {
        FundraiserNotificationObserver {
            observer_id: format!("fundraiser_{}", fundraiser_id),
            fundraiser_id,
        }
    }
}

impl NotificationObserver for FundraiserNotificationObserver {
    fn update(&self, notification: &Notification) {
        // Logic to send notification to fundraiser
        println!("Fundraiser {} received notification: {}", self.fundraiser_id, notification.title);
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
        let fundraiser_id = 123;
        let observer = FundraiserNotificationObserver::new(fundraiser_id);
        let expected_id = format!("fundraiser_{}", fundraiser_id);

        assert_eq!(observer.fundraiser_id, fundraiser_id);
        assert_eq!(observer.observer_id, expected_id);
        assert_eq!(observer.id(), expected_id);
    }

    #[test]
    fn test_update_call() {
        let fundraiser_id = 456;
        let observer = FundraiserNotificationObserver::new(fundraiser_id);

        let notification = Notification {
            id: 10,
            title: "Campaign Update".to_string(),
            content: "Your campaign has been approved.".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::SpecificUser, // Example
            target_id: Some(fundraiser_id), // Example
        };

        // This test primarily ensures the update function can be called
        // without panicking with the current placeholder implementation.
        // More specific tests would require mocking the notification sending logic.
        observer.update(&notification);
        // No panic means the basic call worked.
    }

    // Test Send + Sync bounds (compilation check)
    #[test]
    fn test_thread_safety() {
        let observer = Arc::new(FundraiserNotificationObserver::new(789));
        let observer_clone = observer.clone();

        let notification = Notification {
            id: 11,
            title: "Thread Safety Test".to_string(),
            content: "Testing from another thread.".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::Fundraisers, // Example
            target_id: None, // Example
        };

        let handle = std::thread::spawn(move || {
            // Call update from another thread
            observer_clone.update(&notification);
            // Return the ID from the thread
            observer_clone.id().to_string()
        });

        let returned_id = handle.join().unwrap();

        // Check if the ID is correct from the main thread and the spawned thread
        assert_eq!(observer.id(), "fundraiser_789");
        assert_eq!(returned_id, "fundraiser_789");
    }
}