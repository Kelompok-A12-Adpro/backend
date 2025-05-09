use crate::model::admin::notification::Notification;

pub trait NotificationObserver: Send + Sync {
    fn update(&self, notification: &Notification);
    fn id(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::admin::notification::{Notification, NotificationTargetType};
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    // Mock implementation for testing the trait contract
    struct MockObserver {
        observer_id: String,
        last_notification: Arc<Mutex<Option<Notification>>>,
    }

    impl MockObserver {
        fn new(id: &str) -> Self {
            MockObserver {
                observer_id: id.to_string(),
                last_notification: Arc::new(Mutex::new(None)),
            }
        }

        // Helper to get the last received notification for assertion
        fn get_last_notification(&self) -> Option<Notification> {
            self.last_notification.lock().unwrap().clone()
        }
    }

    impl NotificationObserver for MockObserver {
        fn update(&self, notification: &Notification) {
            let mut last_notif = self.last_notification.lock().unwrap();
            *last_notif = Some(notification.clone());
        }

        fn id(&self) -> &str {
            &self.observer_id
        }
    }

    #[test]
    fn test_observer_id() {
        let observer_id = "test_observer_1";
        let observer = MockObserver::new(observer_id);
        assert_eq!(observer.id(), observer_id);
    }

    #[test]
    fn test_observer_update() {
        let observer = MockObserver::new("observer_update_test");

        // Initially, no notification received
        assert!(observer.get_last_notification().is_none());

        let notification = Notification {
            id: 1,
            title: "Test Update".to_string(),
            content: "Notification content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };

        // Call update
        observer.update(&notification);

        // Check if the notification was stored
        let received_notification = observer.get_last_notification();
        assert!(received_notification.is_some());
        let received = received_notification.unwrap();
        assert_eq!(received.id, notification.id);
        assert_eq!(received.title, notification.title);
        assert_eq!(received.content, notification.content);
        assert_eq!(received.target_type, notification.target_type);
    }

    // Test Send + Sync bounds (compilation check)
    #[test]
    fn test_observer_thread_safety() {
        let observer = Arc::new(MockObserver::new("thread_safe_observer"));
        let observer_clone = observer.clone();

        let notification = Notification {
            id: 2,
            title: "Thread Test".to_string(),
            content: "Sent from thread".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::Fundraisers,
            target_id: None,
        };

        let handle = std::thread::spawn(move || {
            observer_clone.update(&notification);
        });

        handle.join().unwrap();

        // Check if the main observer instance received the update
        let received_notification = observer.get_last_notification();
        assert!(received_notification.is_some());
        assert_eq!(received_notification.unwrap().id, 2);
    }
}