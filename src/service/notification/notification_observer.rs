use std::sync::Arc;
use async_trait::async_trait;
use crate::{errors::AppError, model::admin::notification::{CreateNotificationRequest, Notification}, repository::admin::notification_repo::NotificationRepository};

#[async_trait]
pub trait NotificationObserver: Send + Sync {
    async fn update(&self, request: &CreateNotificationRequest) -> Result<Notification, AppError>;
}

pub struct SubscriberService {
    notification_repo: Arc<dyn NotificationRepository>,
}

impl SubscriberService {
    pub fn new(notification_repo: Arc<dyn NotificationRepository>) -> Self {
        SubscriberService { notification_repo }
    }
}

#[async_trait]
impl NotificationObserver for SubscriberService {
    async fn update(&self, request: &CreateNotificationRequest) -> Result<Notification, AppError> {
        let mut tx = self.notification_repo.begin_transaction().await?;

        match self.notification_repo.create_notification(&request, &mut tx).await {
            Ok(notification) => {
                if let Err(e) = self.notification_repo.push_notification(
                    notification.target_type.clone(),
                    request.adt_detail.clone(),
                    notification.id,
                    &mut tx,
                ).await {
                    return Err(e);
                }
                let _ = tx.commit().await;
                Ok(notification)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::get_test_pool, model::admin::notification::{Notification, NotificationTargetType}};
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    // Mock implementation for testing the trait contract
    struct MockObserver {
        observer_email: String,
        last_notification: Arc<Mutex<Option<(Notification, Option<String>)>>>,
    }

    impl MockObserver {
        fn new(id: &str) -> Self {
            MockObserver {
                observer_email: id.to_string(),
                last_notification: Arc::new(Mutex::new(None)),
            }
        }

        fn get_last_notification(&self) -> Option<(Notification, Option<String>)> {
            self.last_notification.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl NotificationObserver for MockObserver {
        async fn update(&self, request: &CreateNotificationRequest) -> Result<Notification, AppError> {
            let notification = Notification {
                id: 1, // Mock ID for testing
                title: request.title.clone(),
                content: request.content.clone(),
                created_at: Utc::now(),
                target_type: request.target_type.clone(),
            };

            // Simulate storing the notification in the observer
            let adt_detail = request.adt_detail.clone();
            *self.last_notification.lock().unwrap() = Some((notification.clone(), adt_detail));
            Ok(notification)
        }
    }

    // Mock repository for testing SubscriberService
    struct MockNotificationRepository {
        should_fail_create: Arc<Mutex<bool>>,
        should_fail_push: Arc<Mutex<bool>>,
        should_fail_transaction: Arc<Mutex<bool>>,
        created_notifications: Arc<Mutex<Vec<CreateNotificationRequest>>>,
        pushed_notifications: Arc<Mutex<Vec<(NotificationTargetType, Option<String>, i32)>>>,
    }

    impl MockNotificationRepository {
        fn new() -> Self {
            Self {
                should_fail_create: Arc::new(Mutex::new(false)),
                should_fail_push: Arc::new(Mutex::new(false)),
                should_fail_transaction: Arc::new(Mutex::new(false)),
                created_notifications: Arc::new(Mutex::new(Vec::new())),
                pushed_notifications: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn set_fail_create(&self, should_fail: bool) {
            *self.should_fail_create.lock().unwrap() = should_fail;
        }

        fn set_fail_push(&self, should_fail: bool) {
            *self.should_fail_push.lock().unwrap() = should_fail;
        }

        fn set_fail_transaction(&self, should_fail: bool) {
            *self.should_fail_transaction.lock().unwrap() = should_fail;
        }

        fn get_created_notifications(&self) -> Vec<CreateNotificationRequest> {
            self.created_notifications.lock().unwrap().clone()
        }

        fn get_pushed_notifications(&self) -> Vec<(NotificationTargetType, Option<String>, i32)> {
            self.pushed_notifications.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl NotificationRepository for MockNotificationRepository {
        async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, AppError> {
            if self.should_fail_transaction.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock transaction error".to_string()))
            } else {
                get_test_pool().await.begin().await.map_err(|e| AppError::DatabaseError(e.to_string()))
            }
        }
        
        async fn create_notification(
            &self,
            request: &CreateNotificationRequest,
            tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<Notification, AppError> {
            if self.should_fail_create.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock create error".to_string()))
            } else {
                self.created_notifications.lock().unwrap().push(request.clone());
                Ok(Notification {
                    id: 1,
                    title: request.title.clone(),
                    content: request.content.clone(),
                    created_at: Utc::now(),
                    target_type: request.target_type.clone(),
                })
            }
        }

        async fn push_notification(
            &self,
            target: NotificationTargetType,
            adt_details: Option<String>,
            notification_id: i32,
            tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<bool, AppError> {
            if self.should_fail_push.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock push error".to_string()))
            } else {
                self.pushed_notifications.lock().unwrap().push((target, adt_details, notification_id));
                Ok(true)
            }
        }

        async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError> {
            if self.should_fail_transaction.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock error".to_string()))
            } else {
                Ok(vec![])
            }
        }

        async fn get_notification_for_user(
            &self,
            _user_email: String,
        ) -> Result<Vec<Notification>, AppError> {
            if self.should_fail_transaction.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock error".to_string()))
            } else {
                Ok(vec![])
            }
        }

        async fn get_notification_by_id(
            &self,
            _notification_id: i32,
        ) -> Result<Option<Notification>, AppError> {
            if self.should_fail_transaction.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock error".to_string()))
            } else {
                Ok(None)
            }
        }

        async fn delete_notification(&self, _notification_id: i32) -> Result<bool, AppError> {
            if self.should_fail_transaction.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock error".to_string()))
            } else {
                Ok(true)
            }
        }

        async fn delete_notification_user(
            &self,
            _notification_id: i32,
            _user_email: String,
        ) -> Result<bool, AppError> {
            if self.should_fail_transaction.lock().unwrap().clone() {
                Err(AppError::DatabaseError("Mock error".to_string()))
            } else {
                Ok(true)
            }
        }
    }

    #[tokio::test]
    async fn test_observer_update() {
        let observer = MockObserver::new("observer_update_test");

        assert!(observer.get_last_notification().is_none());

        let notification = Notification {
            id: 1,
            title: "Test Update".to_string(),
            content: "Notification content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };

        let adt_detail = Some("test_detail".to_string());

        observer.update(&CreateNotificationRequest {
            title: notification.title.clone(),
            content: notification.content.clone(),
            target_type: notification.target_type.clone(),
            adt_detail: adt_detail.clone(),
        }).await.unwrap();

        let received = observer.get_last_notification();
        assert!(received.is_some());
        let (received_notification, received_detail) = received.unwrap();
        assert_eq!(received_notification.id, notification.id);
        assert_eq!(received_notification.title, notification.title);
        assert_eq!(received_notification.content, notification.content);
        assert_eq!(received_notification.target_type, notification.target_type);
        assert_eq!(received_detail, adt_detail);
    }

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
        };

        let handle = std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                observer_clone.update(&CreateNotificationRequest {
                    title: notification.title.clone(),
                    content: notification.content.clone(),
                    target_type: notification.target_type.clone(),
                    adt_detail: None,
                }).await.unwrap();
            });
        });

        handle.join().unwrap();

        let received_notification = observer.get_last_notification();
        assert!(received_notification.is_some());
        assert_eq!(received_notification.unwrap().0.id, 2);
    }

    #[tokio::test]
    async fn test_subscriber_service_new() {
        let mock_repo = Arc::new(MockNotificationRepository::new());
        let _ = SubscriberService::new(mock_repo.clone());
        
        // Verify service was created with the repository
        assert_eq!(Arc::strong_count(&mock_repo), 2); // service + our reference
    }

    #[tokio::test]
    async fn test_subscriber_service_update_success() {
        let mock_repo = Arc::new(MockNotificationRepository::new());
        let service = SubscriberService::new(mock_repo.clone());

        let notification = Notification {
            id: 1,
            title: "Test Notification".to_string(),
            content: "Test content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };

        let adt_detail = Some("test_detail".to_string());

        let result = service.update(&CreateNotificationRequest {
            title: notification.title.clone(),
            content: notification.content.clone(),
            target_type: notification.target_type.clone(),
            adt_detail: None,
        }).await;

        assert!(result.is_ok());

        let created = mock_repo.get_created_notifications();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].title, notification.title);
        assert_eq!(created[0].content, notification.content);
        assert_eq!(created[0].target_type, notification.target_type);
        assert_eq!(created[0].adt_detail, adt_detail);

        let pushed = mock_repo.get_pushed_notifications();
        assert_eq!(pushed.len(), 1);
        assert_eq!(pushed[0].0, notification.target_type);
        assert_eq!(pushed[0].1, adt_detail);
        assert_eq!(pushed[0].2, 1);
    }

    #[tokio::test]
    async fn test_subscriber_service_update_without_adt_detail() {
        let mock_repo = Arc::new(MockNotificationRepository::new());
        let service = SubscriberService::new(mock_repo.clone());

        let notification = Notification {
            id: 1,
            title: "Test Notification".to_string(),
            content: "Test content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::Fundraisers,
        };

        let result = service.update(&CreateNotificationRequest {
            title: notification.title.clone(),
            content: notification.content.clone(),
            target_type: notification.target_type.clone(),
            adt_detail: None,
        }).await;

        assert!(result.is_ok());

        let created = mock_repo.get_created_notifications();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].adt_detail, None);

        let pushed = mock_repo.get_pushed_notifications();
        assert_eq!(pushed.len(), 1);
        assert_eq!(pushed[0].1, None);
    }

    #[tokio::test]
    async fn test_subscriber_service_update_create_notification_fails() {
        let mock_repo = Arc::new(MockNotificationRepository::new());
        mock_repo.set_fail_create(true);
        let service = SubscriberService::new(mock_repo.clone());

        let notification = Notification {
            id: 1,
            title: "Test Notification".to_string(),
            content: "Test content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };

        let result = service.update(&CreateNotificationRequest {
            title: notification.title.clone(),
            content: notification.content.clone(),
            target_type: notification.target_type.clone(),
            adt_detail: None,
        }).await;

        assert!(result.is_err());
        
        if let Err(AppError::DatabaseError(msg)) = result {
            assert_eq!(msg, "Mock create error");
        } else {
            panic!("Expected DatabaseError");
        }

        // Ensure no notifications were pushed when create fails
        let pushed = mock_repo.get_pushed_notifications();
        assert_eq!(pushed.len(), 0);
    }

    #[tokio::test]
    async fn test_subscriber_service_update_push_notification_fails() {
        let mock_repo = Arc::new(MockNotificationRepository::new());
        mock_repo.set_fail_push(true);
        let service = SubscriberService::new(mock_repo.clone());

        let notification = Notification {
            id: 1,
            title: "Test Notification".to_string(),
            content: "Test content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };

        let result = service.update(&CreateNotificationRequest {
            title: notification.title.clone(),
            content: notification.content.clone(),
            target_type: notification.target_type.clone(),
            adt_detail: None,
        }).await;

        assert!(result.is_err());
        
        if let Err(AppError::DatabaseError(msg)) = result {
            assert_eq!(msg, "Mock push error");
        } else {
            panic!("Expected DatabaseError");
        }

        // Ensure notification was created but push failed
        let created = mock_repo.get_created_notifications();
        assert_eq!(created.len(), 1);
    }

    #[tokio::test]
    async fn test_subscriber_service_update_transaction_begin_fails() {
        let mock_repo = Arc::new(MockNotificationRepository::new());
        mock_repo.set_fail_transaction(true);
        let service = SubscriberService::new(mock_repo.clone());

        let notification = Notification {
            id: 1,
            title: "Test Notification".to_string(),
            content: "Test content".to_string(),
            created_at: Utc::now(),
            target_type: NotificationTargetType::AllUsers,
        };

        let result = service.update(&CreateNotificationRequest {
            title: notification.title.clone(),
            content: notification.content.clone(),
            target_type: notification.target_type.clone(),
            adt_detail: None,
        }).await;

        assert!(result.is_err());
        
        if let Err(AppError::DatabaseError(msg)) = result {
            assert_eq!(msg, "Mock transaction error");
        } else {
            panic!("Expected DatabaseError");
        }
    }
}