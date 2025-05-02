use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::admin::notification::{AdminNotificationView, NotificationTargetType, CreateNotificationRequest};

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create_notification(&self, notification: &CreateNotificationRequest) -> Result<AdminNotificationView, AppError>;
    async fn get_all_notifications(&self) -> Result<Vec<AdminNotificationView>, AppError>;
    async fn get_notification_by_id(&self, notification_id: i32) -> Result<Option<AdminNotificationView>, AppError>;
    async fn delete_notification(&self, notification_id: i32) -> Result<bool, AppError>;
}

// Implementation will be added later
