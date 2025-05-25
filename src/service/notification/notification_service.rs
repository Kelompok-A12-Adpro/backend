use std::sync::{Arc, Mutex};
use crate::errors::AppError;
use crate::service::notification::notification_observer::NotificationObserver;
use crate::model::admin::notification::{Notification, CreateNotificationRequest};
use chrono::Utc;

pub struct NotificationService {
    observers: Mutex<Vec<Box<dyn NotificationObserver>>>,
    notifications: Mutex<Vec<Notification>>,
    next_id: Mutex<i32>,
}

impl NotificationService {
    pub fn new() -> Self {
        NotificationService {
            observers: Mutex::new(Vec::new()),
            notifications: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
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
        if command.title.is_empty() || command.content.is_empty() {
            return Err(AppError::ValidationError("Title and content cannot be empty".to_string()));
        }
        
        let mut notifications = self.notifications.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        
        let new_notification = Notification {
            id: *next_id,
            title: command.title,
            content: command.content,
            created_at: Utc::now(),
            target_type: command.target_type,
            target_id: command.target_id,
        };
        
        notifications.push(new_notification.clone());
        *next_id += 1;
        
        drop(notifications);
        drop(next_id);

        self.notify_all(&new_notification);
        
        Ok(new_notification)
    }
    
    pub async fn delete_notification(&self, notification_id: i32) -> Result<(), AppError> {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.retain(|n| n.id != notification_id);
        Ok(())
    }
    
    pub async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError> {
        let notifications = self.notifications.lock().unwrap();
        Ok(notifications.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::model::admin::new_campaign_subs::NewCampaignSubscription;
    use crate::model::admin::notification::{
        CreateNotificationRequest, Notification, NotificationTargetType,
    };
    use crate::service::notification::notification_observer::NotificationObserver;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};
    use async_trait::async_trait;

    struct MockObserver {
        id: String,
        update_called: Arc<Mutex<bool>>,
        last_request: Arc<Mutex<Option<CreateNotificationRequest>>>,
        should_fail: Arc<Mutex<bool>>,
        notifications_created: Arc<Mutex<Vec<Notification>>>,
        next_id: Arc<Mutex<i32>>,
    }

    impl MockObserver {
        fn new(id: &str) -> Self {
            MockObserver {
                id: id.to_string(),
                update_called: Arc::new(Mutex::new(false)),
                last_request: Arc::new(Mutex::new(None)),
                should_fail: Arc::new(Mutex::new(false)),
                notifications_created: Arc::new(Mutex::new(Vec::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }

        fn was_update_called(&self) -> bool {
            *self.update_called.lock().unwrap()
        }

        fn reset(&self) {
            *self.update_called.lock().unwrap() = false;
            *self.last_request.lock().unwrap() = None;
        }

        fn get_last_request(&self) -> Option<CreateNotificationRequest> {
            self.last_request.lock().unwrap().clone()
        }

        fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }

        fn get_created_notifications(&self) -> Vec<Notification> {
            self.notifications_created.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl NotificationObserver for MockObserver {
        async fn update(&self, request: &CreateNotificationRequest) -> Result<Notification, AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::ValidationError("Observer failed".to_string()));
            }

            *self.update_called.lock().unwrap() = true;
            *self.last_request.lock().unwrap() = Some(request.clone());
            
            let mut next_id = self.next_id.lock().unwrap();
            let notification = Notification {
                id: *next_id,
                title: request.title.clone(),
                content: request.content.clone(),
                created_at: Utc::now(),
                target_type: request.target_type.clone(),
            };
            *next_id += 1;

            self.notifications_created.lock().unwrap().push(notification.clone());
            Ok(notification)
        }
    }

    // Mock Subscription Repository
    struct MockNewCampaignSubscriptionRepository {
        should_fail: Arc<Mutex<bool>>,
        subscribers: Arc<Mutex<Vec<String>>>,
    }

    impl MockNewCampaignSubscriptionRepository {
        fn new() -> Self {
            Self {
                should_fail: Arc::new(Mutex::new(false)),
                subscribers: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }

        fn get_subscribers_count(&self) -> usize {
            self.subscribers.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl NewCampaignSubscriptionRepository for MockNewCampaignSubscriptionRepository {
        async fn subscribe(&self, user_email: String) -> Result<(), AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::DatabaseError("Subscribe failed".to_string()));
            }
            self.subscribers.lock().unwrap().push(user_email);
            Ok(())
        }

        async fn unsubscribe(&self, user_email: String) -> Result<(), AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::DatabaseError("Unsubscribe failed".to_string()));
            }
            self.subscribers.lock().unwrap().retain(|email| email != &user_email);
            Ok(())
        }

        async fn get_subscribers(&self) -> Result<Vec<NewCampaignSubscription>, AppError> {
            Ok(Vec::new())
        }
    }

    // Mock Notification Repository
    struct MockNotificationRepository {
        notifications: Arc<Mutex<Vec<Notification>>>,
        should_fail: Arc<Mutex<bool>>,
    }

    impl MockNotificationRepository {
        fn new() -> Self {
            Self {
                notifications: Arc::new(Mutex::new(Vec::new())),
                should_fail: Arc::new(Mutex::new(false)),
            }
        }

        fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }

        fn add_notification(&self, notification: Notification) {
            self.notifications.lock().unwrap().push(notification);
        }
    }

    #[async_trait]
    impl NotificationRepository for MockNotificationRepository {
        async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, AppError> {
            Err(AppError::DatabaseError("Not implemented for mock".to_string()))
        }

        async fn create_notification(
            &self,
            _request: &CreateNotificationRequest,
            _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<Notification, AppError> {
            Err(AppError::DatabaseError("Not implemented for mock".to_string()))
        }

        async fn push_notification(
            &self,
            _target: NotificationTargetType,
            _adt_details: Option<String>,
            _notification_id: i32,
            _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<bool, AppError> {
            Err(AppError::DatabaseError("Not implemented for mock".to_string()))
        }

        async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::DatabaseError("Get all failed".to_string()));
            }
            Ok(self.notifications.lock().unwrap().clone())
        }

        async fn get_notification_for_user(&self, _user_email: String) -> Result<Vec<Notification>, AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::DatabaseError("Get for user failed".to_string()));
            }
            Ok(self.notifications.lock().unwrap().clone())
        }

        async fn get_notification_by_id(&self, notification_id: i32) -> Result<Option<Notification>, AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::DatabaseError("Get by id failed".to_string()));
            }
            let notifications = self.notifications.lock().unwrap();
            Ok(notifications.iter().find(|n| n.id == notification_id).cloned())
        }

        async fn delete_notification(&self, notification_id: i32) -> Result<bool, AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::DatabaseError("Delete failed".to_string()));
            }
            let mut notifications = self.notifications.lock().unwrap();
            let initial_len = notifications.len();
            notifications.retain(|n| n.id != notification_id);
            Ok(notifications.len() < initial_len)
        }

        async fn delete_notification_user(&self, _notification_id: i32, _user_email: String) -> Result<bool, AppError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AppError::DatabaseError("Delete user failed".to_string()));
            }
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_service_creation() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        // Service should be created successfully
        assert_eq!(service.notification_repo.get_all_notifications().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_subscribe_success() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo.clone(), observer);
        
        let result = service.subscribe("user@test.com".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(subscriber_repo.get_subscribers_count(), 1);
    }

    #[tokio::test]
    async fn test_subscribe_validation_error() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.subscribe("".to_string()).await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "User email cannot be empty");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_subscribe_repository_failure() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        subscriber_repo.set_should_fail(true);
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.subscribe("user@test.com".to_string()).await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::DatabaseError(msg) => {
                assert!(msg.contains("Failed to subscribe user"));
            }
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[tokio::test]
    async fn test_unsubscribe_success() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo.clone(), observer);
        
        // First subscribe
        service.subscribe("user@test.com".to_string()).await.unwrap();
        assert_eq!(subscriber_repo.get_subscribers_count(), 1);
        
        // Then unsubscribe
        let result = service.unsubscribe("user@test.com".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(subscriber_repo.get_subscribers_count(), 0);
    }

    #[tokio::test]
    async fn test_unsubscribe_validation_error() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.unsubscribe("".to_string()).await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "User email cannot be empty");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_notify_success() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer.clone());
        
        let request = CreateNotificationRequest {
            title: "Test Notification".to_string(),
            content: "Test content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };
        
        let result = service.notify(request.clone()).await;
        assert!(result.is_ok());
        
        // Check observer was called
        assert!(observer.was_update_called());
        
        let last_request = observer.get_last_request();
        assert!(last_request.is_some());
        assert_eq!(last_request.unwrap().title, request.title);
    }

    #[tokio::test]
    async fn test_notify_observer_failure() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        observer.set_should_fail(true);
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let request = CreateNotificationRequest {
            title: "Test Notification".to_string(),
            content: "Test content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };
        
        let result = service.notify(request).await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "Observer failed");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_get_all_notifications() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo.clone(), subscriber_repo, observer);
        
        // Add some test notifications
        let notification1 = Notification {
            id: 1,
            title: "First".to_string(),
            content: "Content 1".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };
        
        let notification2 = Notification {
            id: 2,
            title: "Second".to_string(),
            content: "Content 2".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };
        
        repo.add_notification(notification1);
        repo.add_notification(notification2);
        
        let notifications = service.get_all_notifications().await.unwrap();
        assert_eq!(notifications.len(), 2);
    }

    #[tokio::test]
    async fn test_get_all_notifications_repository_failure() {
        let repo = Arc::new(MockNotificationRepository::new());
        repo.set_should_fail(true);
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.get_all_notifications().await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::DatabaseError(msg) => {
                assert_eq!(msg, "Get all failed");
            }
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[tokio::test]
    async fn test_get_notification_by_id() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo.clone(), subscriber_repo, observer);
        
        let notification = Notification {
            id: 1,
            title: "Test".to_string(),
            content: "Content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };
        
        repo.add_notification(notification.clone());
        
        let found = service.get_notification_by_id(1).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
        
        let not_found = service.get_notification_by_id(999).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_notifications_for_user() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo.clone(), subscriber_repo, observer);
        
        let notification = Notification {
            id: 1,
            title: "User Notification".to_string(),
            content: "For user".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::SpecificUser,
        };
        
        repo.add_notification(notification);
        
        let notifications = service.get_notifications_for_user("user@test.com".to_string()).await.unwrap();
        assert_eq!(notifications.len(), 1);
    }

    #[tokio::test]
    async fn test_get_notifications_for_user_validation() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.get_notifications_for_user("".to_string()).await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "User email cannot be empty");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_delete_notification() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo.clone(), subscriber_repo, observer);
        
        let notification = Notification {
            id: 1,
            title: "To Delete".to_string(),
            content: "Delete me".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };
        
        repo.add_notification(notification);
        
        let deleted = service.delete_notification(1).await.unwrap();
        assert!(deleted);
        
        let not_found = service.get_notification_by_id(1).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_delete_notification_validation() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.delete_notification(0).await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "Invalid notification ID");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_delete_notification_for_user() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.delete_notification_for_user(1, "user@test.com".to_string()).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_delete_notification_for_user_validation() {
        let repo = Arc::new(MockNotificationRepository::new());
        let subscriber_repo = Arc::new(MockNewCampaignSubscriptionRepository::new());
        let observer = Arc::new(MockObserver::new("test_observer"));
        let service = NotificationService::new(repo, subscriber_repo, observer);
        
        let result = service.delete_notification_for_user(0, "user@test.com".to_string()).await;
        assert!(result.is_err());
        
        let result = service.delete_notification_for_user(1, "".to_string()).await;
        assert!(result.is_err());
        
        match result.err().unwrap() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "User email cannot be empty");
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
