use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::admin::notification::{Notification, NotificationTargetType, CreateNotificationRequest};

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create_notification(&self, notification: &CreateNotificationRequest) -> Result<Notification, AppError>;
    async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError>;
    async fn get_notification_by_id(&self, notification_id: i32) -> Result<Option<Notification>, AppError>;
    async fn delete_notification(&self, notification_id: i32) -> Result<bool, AppError>;
}

// Implementation will be added later

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::admin::notification::NotificationTargetType;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tokio; // Ensure tokio is a dev-dependency for async tests

    // Mock implementation for testing the trait contract
    struct MockNotificationRepository {
        notifications: Arc<Mutex<HashMap<i32, Notification>>>,
        next_id: Arc<Mutex<i32>>,
    }

    impl MockNotificationRepository {
        fn new() -> Self {
            MockNotificationRepository {
                notifications: Arc::new(Mutex::new(HashMap::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }
    }

    #[async_trait]
    impl NotificationRepository for MockNotificationRepository {
        async fn create_notification(&self, request: &CreateNotificationRequest) -> Result<Notification, AppError> {
            let mut notifications = self.notifications.lock().unwrap();
            let mut next_id_guard = self.next_id.lock().unwrap();
            let id = *next_id_guard;
            *next_id_guard += 1;

            let notification = Notification {
                id,
                title: request.title.clone(),
                content: request.content.clone(),
                created_at: Utc::now(),
                target_type: request.target_type.clone(),
                target_id: request.target_id,
            };
            notifications.insert(id, notification.clone());
            Ok(notification)
        }

        async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError> {
            let notifications = self.notifications.lock().unwrap();
            Ok(notifications.values().cloned().collect())
        }

        async fn get_notification_by_id(&self, notification_id: i32) -> Result<Option<Notification>, AppError> {
            let notifications = self.notifications.lock().unwrap();
            Ok(notifications.get(&notification_id).cloned())
        }

        async fn delete_notification(&self, notification_id: i32) -> Result<bool, AppError> {
            let mut notifications = self.notifications.lock().unwrap();
            Ok(notifications.remove(&notification_id).is_some())
        }
    }

    #[tokio::test]
    async fn test_create_and_get_notification() {
        let repo = MockNotificationRepository::new();
        let request = CreateNotificationRequest {
            title: "Test Notification".to_string(),
            content: "This is a test.".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };

        let created_notification = repo.create_notification(&request).await.expect("Failed to create notification");
        assert_eq!(created_notification.title, request.title);
        assert_eq!(created_notification.content, request.content);
        assert_eq!(created_notification.target_type, request.target_type);
        assert!(created_notification.id > 0); // Should have a positive ID

        let fetched_notification = repo.get_notification_by_id(created_notification.id).await.expect("Failed to get notification");
        assert!(fetched_notification.is_some());
        assert_eq!(fetched_notification.unwrap().id, created_notification.id);
    }

    #[tokio::test]
    async fn test_get_non_existent_notification() {
        let repo = MockNotificationRepository::new();
        let fetched_notification = repo.get_notification_by_id(999).await.expect("Failed to get notification");
        assert!(fetched_notification.is_none());
    }

    #[tokio::test]
    async fn test_get_all_notifications() {
        let repo = MockNotificationRepository::new();
        let request1 = CreateNotificationRequest {
            title: "Notification 1".to_string(),
            content: "Content 1".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };
        let request2 = CreateNotificationRequest {
            title: "Notification 2".to_string(),
            content: "Content 2".to_string(),
            target_type: NotificationTargetType::Fundraisers,
            target_id: None,
        };

        repo.create_notification(&request1).await.unwrap();
        repo.create_notification(&request2).await.unwrap();

        let all_notifications = repo.get_all_notifications().await.expect("Failed to get all notifications");
        assert_eq!(all_notifications.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_notification() {
        let repo = MockNotificationRepository::new();
        let request = CreateNotificationRequest {
            title: "To Be Deleted".to_string(),
            content: "Delete me".to_string(),
            target_type: NotificationTargetType::AllUsers,
            target_id: None,
        };

        let created_notification = repo.create_notification(&request).await.unwrap();
        let notification_id = created_notification.id;

        // Ensure it exists
        assert!(repo.get_notification_by_id(notification_id).await.unwrap().is_some());

        // Delete it
        let delete_result = repo.delete_notification(notification_id).await.expect("Failed to delete notification");
        assert!(delete_result); // Should return true for successful deletion

        // Ensure it's gone
        assert!(repo.get_notification_by_id(notification_id).await.unwrap().is_none());

        // Try deleting again (should fail)
        let delete_again_result = repo.delete_notification(notification_id).await.expect("Failed to delete non-existent notification");
        assert!(!delete_again_result); // Should return false
    }
}