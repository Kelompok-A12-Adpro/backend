#[cfg(test)]
mod tests {
    use crate::service::campaign::observer::campaign_observer::{CampaignObserver, CampaignNotifier};
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use std::sync::{Arc, Mutex};
    use chrono::Utc;

    // Mock observer that counts notifications
    struct MockObserver {
        notifications: Arc<Mutex<Vec<String>>>,
    }
    
    impl MockObserver {
        fn new() -> Self {
            MockObserver {
                notifications: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn notification_count(&self) -> usize {
            self.notifications.lock().unwrap().len()
        }
        
        fn last_notification(&self) -> Option<String> {
            let notifications = self.notifications.lock().unwrap();
            notifications.last().cloned()
        }
    }
    
    impl CampaignObserver for MockObserver {
        fn on_campaign_status_changed(&self, campaign: &Campaign, old_status: CampaignStatus) {
            let msg = format!(
                "Campaign {} status changed from {:?} to {:?}", 
                campaign.id, old_status, campaign.status
            );
            self.notifications.lock().unwrap().push(msg);
        }
    }

    #[test]
    fn test_observer_notification() {
        let observer = Arc::new(MockObserver::new());
        let mut notifier = CampaignNotifier::new();
        
        // Register observer
        notifier.attach(observer.clone());
        
        // Create test campaign
        let mut campaign = Campaign {
            id: 1,
            user_id: 10,
            name: String::from("Test Campaign"),
            description: String::from("Test Description"),
            target_amount: 1000.0,
            collected_amount: 0.0,
            status: CampaignStatus::PendingVerification,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        };
        
        // Verify no notifications initially
        assert_eq!(observer.notification_count(), 0);
        
        // Update campaign status
        let old_status = campaign.status.clone();
        campaign.status = CampaignStatus::Active;
        
        // Notify observers about the change
        notifier.notify_status_change(&campaign, old_status);
        
        // Verify notification was received
        assert_eq!(observer.notification_count(), 1);
        assert!(observer.last_notification().unwrap().contains("status changed from PendingVerification to Active"));
    }
}