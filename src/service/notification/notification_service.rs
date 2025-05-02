use std::sync::{Arc, Mutex};
use crate::errors::AppError;
use crate::service::notification::notification_observer::NotificationObserver;
use crate::model::admin::notification::{Notification, CreateNotificationRequest};

pub struct NotificationService {
    observers: Mutex<Vec<Box<dyn NotificationObserver>>>,
}

impl NotificationService {
    pub fn new() -> Self {
        NotificationService {
            observers: Mutex::new(Vec::new()),
        }
    }
    
    pub fn attach(&self, observer: Box<dyn NotificationObserver>) {
        let mut observers = self.observers.lock().unwrap();
        observers.push(observer);
    }
    
    pub fn detach(&self, observer_id: &str) {
        let mut observers = self.observers.lock().unwrap();
        observers.retain(|obs| obs.id() != observer_id);
    }
    
    pub fn notify_all(&self, notification: &Notification) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.update(notification);
        }
    }
    
    pub async fn create_notification(&self, command: CreateNotificationRequest) -> Result<Notification, AppError> {
        // Validate input
        if command.title.is_empty() || command.content.is_empty() {
            return Err(AppError::ValidationError("Title and content cannot be empty".to_string()));
        }
        
        // Create notification logic would go here
        unimplemented!()
    }
    
    pub async fn delete_notification(&self, notification_id: i32) -> Result<(), AppError> {
        // Delete notification logic would go here
        unimplemented!()
    }
    
    pub async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError> {
        // Get all notifications logic would go here
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::admin::notification::{Notification, NotificationTargetType, CreateNotificationRequest};
    use crate::service::notification::notification_observer::NotificationObserver;
    use crate::errors::AppError;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};
    use tokio; // Ensure tokio is a dev-dependency

    // Mock Observer for testing attach, detach, notify
    struct MockObserver {
        id: String,
        update_called: Arc<Mutex<bool>>,
        last_notification_id: Arc<Mutex<Option<i32>>>,
    }

    impl MockObserver {
        fn new(id: &str) -> Self {
            MockObserver {
                id: id.to_string(),
                update_called: Arc::new(Mutex::new(false)),
                last_notification_id: Arc::new(Mutex::new(None)),
            }
        }

        fn was_update_called(&self) -> bool {
            *self.update_called.lock().unwrap()
        }

        fn reset(&self) {
            *self.update_called.lock().unwrap() = false;
            *self.last_notification_id.lock().unwrap() = None;
        }

         fn get_last_notification_id(&self) -> Option<i32> {
            *self.last_notification_id.lock().unwrap()
        }
    }

    impl NotificationObserver for MockObserver {
        fn update(&self, notification: &Notification) {
            *self.update_called.lock().unwrap() = true;
            *self.last_notification_id.lock().unwrap() = Some(notification.id);
        }

        fn id(&self) -> &str {
            &self.id
        }
    }

    #[test]
    fn test_attach_and_notify() {
        let service = NotificationService::new();
        let observer1 = MockObserver::new("obs1");
        let observer1_update_flag = observer1.update_called.clone();

        service.attach(Box::new(observer1));

        let notification = Notification {
            id: 1, title: "N1".to_string(), content: "C1".to_string(), created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers, target_id: None
        };

        assert!(!*observer1_update_flag.lock().unwrap(), "Observer should not have been updated yet");
        service.notify_all(&notification);
        assert!(*observer1_update_flag.lock().unwrap(), "Observer should have been updated");
    }

    #[test]
    fn test_detach() {
        let service = NotificationService::new();
        let observer1 = MockObserver::new("obs1");
        let observer2 = MockObserver::new("obs2");
        let observer1_update_flag = observer1.update_called.clone();
        let observer2_update_flag = observer2.update_called.clone();

        service.attach(Box::new(observer1));
        service.attach(Box::new(observer2));

        // Detach observer1
        service.detach("obs1");

        let notification = Notification {
            id: 2, title: "N2".to_string(), content: "C2".to_string(), created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers, target_id: None
        };

        service.notify_all(&notification);

        assert!(!*observer1_update_flag.lock().unwrap(), "Detached observer 1 should not be updated");
        assert!(*observer2_update_flag.lock().unwrap(), "Observer 2 should be updated");
    }

    #[test]
    fn test_notify_all_multiple_observers() {
        let service = NotificationService::new();
        let observer1 = MockObserver::new("obs1");
        let observer2 = MockObserver::new("obs2");
        let observer1_update_flag = observer1.update_called.clone();
        let observer2_update_flag = observer2.update_called.clone();
        let observer1_last_id = observer1.last_notification_id.clone();
        let observer2_last_id = observer2.last_notification_id.clone();


        service.attach(Box::new(observer1));
        service.attach(Box::new(observer2));

        let notification = Notification {
            id: 3, title: "N3".to_string(), content: "C3".to_string(), created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers, target_id: None
        };

        service.notify_all(&notification);

        assert!(*observer1_update_flag.lock().unwrap(), "Observer 1 should be updated");
        assert!(*observer2_update_flag.lock().unwrap(), "Observer 2 should be updated");
        assert_eq!(*observer1_last_id.lock().unwrap(), Some(3));
        assert_eq!(*observer2_last_id.lock().unwrap(), Some(3));
    }

    #[tokio::test]
    async fn test_create_notification_validation_empty_title() {
        let service = NotificationService::new();
        let command = CreateNotificationRequest {
            title: "".to_string(),
            content: "Some content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };

        let result = service.create_notification(command).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::ValidationError(msg) => assert_eq!(msg, "Title and content cannot be empty"),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_create_notification_validation_empty_content() {
        let service = NotificationService::new();
        let command = CreateNotificationRequest {
            title: "Some Title".to_string(),
            content: "".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };

        let result = service.create_notification(command).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::ValidationError(msg) => assert_eq!(msg, "Title and content cannot be empty"),
            _ => panic!("Expected ValidationError"),
        }
    }
}