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
