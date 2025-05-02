use crate::service::notification::notification_observer::NotificationObserver;
use crate::service::notification::models::Notification;

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
