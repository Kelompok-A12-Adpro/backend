use std::sync::Arc;
use crate::model::campaign::campaign::{Campaign, CampaignStatus};

pub trait CampaignObserver: Send + Sync {
    fn on_campaign_status_changed(&self, campaign: &Campaign, old_status: CampaignStatus);
}

pub struct CampaignNotifier {
    observers: Vec<Arc<dyn CampaignObserver>>,
}

impl CampaignNotifier {
    pub fn new() -> Self {
        CampaignNotifier {
            observers: Vec::new(),
        }
    }
    
    pub fn attach(&mut self, observer: Arc<dyn CampaignObserver>) {
        self.observers.push(observer);
    }
    
    pub fn detach(&mut self, observer: &Arc<dyn CampaignObserver>) {
        if let Some(index) = self.observers.iter().position(|o| Arc::ptr_eq(o, observer)) {
            self.observers.remove(index);
        }
    }
    
    pub fn notify_status_change(&self, campaign: &Campaign, old_status: CampaignStatus) {
        for observer in &self.observers {
            observer.on_campaign_status_changed(campaign, old_status.clone());
        }
    }
}