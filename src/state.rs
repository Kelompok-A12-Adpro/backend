use std::sync::Arc;
use rocket::{Rocket, Build};
use sqlx::PgPool;

use crate::repository::admin::statistics_repo::DbStatisticsRepository;
use crate::repository::campaign::campaign_repository::InMemoryCampaignRepository;
use crate::repository::donation::donation_repository::PgDonationRepository;
use crate::service::admin::platform_statistics_service::PlatformStatisticsService;
use crate::service::donation::donation_service::DonationService;
// TODO: Import other repositories if yours need shared state

pub struct AppState {
    pub donation_service: DonationService,
    pub dashboard_service: PlatformStatisticsService
    // TODO: Import other services if yours need shared state
}

pub async fn init_state(pool: PgPool) -> AppState {
    // TODO: Initialize other services if yours need shared state

    // Repos
    let donation_repo = Arc::new(PgDonationRepository::new(pool.clone()));
    let campaign_repo = Arc::new(InMemoryCampaignRepository::new());
    let dashboard_repo = Arc::new(DbStatisticsRepository::new());

    // Services
    let donation_service = DonationService::new(donation_repo, campaign_repo);
    let dashboard_service = PlatformStatisticsService::new(dashboard_repo);
    
    AppState {
        donation_service,
        dashboard_service,
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
            .manage(state.dashboard_service)
    }
}