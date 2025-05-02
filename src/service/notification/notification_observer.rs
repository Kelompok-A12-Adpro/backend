use crate::model::admin::notification::Notification;

pub trait NotificationObserver: Send + Sync {
    fn update(&self, notification: &Notification);
    fn id(&self) -> &str;
}
