use std::sync::Arc;
use rocket::{Rocket, Build};
use sqlx::PgPool;

use crate::repository::campaign::campaign_repository::InMemoryCampaignRepository;
use crate::repository::donation::donation_repository::PgDonationRepository;
use crate::service::donation::donation_service::DonationService;
// TODO: Import other repositories if yours need shared state

pub struct AppState {
    pub donation_service: DonationService,
    // TODO: Import other services if yours need shared state
}

pub async fn init_state(pool: PgPool) -> AppState {
    // TODO: Initialize other services if yours need shared state

    // Repos
    let donation_repo = Arc::new(PgDonationRepository::new(pool.clone()));
    let campaign_repo = Arc::new(InMemoryCampaignRepository::new());

    // Services
    let donation_service = DonationService::new(donation_repo, campaign_repo);
    
    AppState {
        donation_service,
    }
}

/// Add all managed state to the Rocket instance
pub trait StateManagement {
    fn manage_state(self, state: AppState) -> Self;
}

impl StateManagement for Rocket<Build> {
    fn manage_state(self, state: AppState) -> Self {
        // TODO: Add other services
        self.manage(state.donation_service)
    }
}