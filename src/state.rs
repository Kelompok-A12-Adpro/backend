use std::sync::Arc;
use rocket::{Rocket, Build};
use sqlx::PgPool;

use crate::repository::admin::notification_repo::DbNotificationRepository;
use crate::repository::admin::new_campaign_subs_repo::DbNewCampaignSubscriptionRepository;
// This import is correct and brings CampaignTotalsCache into scope
use crate::repository::donation::donation_repository::{
    PgDonationRepository, 
    CampaignTotalsCache, 
    UserCampaignDonationCache
}; 
use crate::repository::campaign::campaign_repository::PgCampaignRepository;


use crate::repository::wallet::transaction_repository::PgTransactionRepository;
use crate::repository::wallet::wallet_repository::PgWalletRepository;
use crate::service::campaign::factory::campaign_factory::CampaignFactory;
use crate::service::admin::notification::notification_observer::SubscriberService;

use crate::service::admin::notification::notification_service::NotificationService;
use crate::service::admin::platform_statistics_service::StatisticService;
use crate::service::donation::donation_service::DonationService;
use crate::service::campaign::campaign_service::CampaignService;
use crate::service::wallet::wallet_service::WalletService;

// Your AppState struct
pub struct AppState {
    pub donation_service: DonationService,
    pub campaign_service: Arc<CampaignService>,
    pub campaign_factory: Arc<CampaignFactory>,
    pub notification_service: Arc<NotificationService>,
    pub statistic_service: Arc<StatisticService>,
    pub wallet_service: WalletService
}

// MODIFIED FUNCTION SIGNATURE:
pub async fn init_state(
    pool: PgPool, 
    campaign_totals_cache: CampaignTotalsCache, 
    user_campaign_donation_cache: UserCampaignDonationCache
) -> AppState {
    // Repos
    // Now `campaign_totals_cache` is available from the function parameters:
    let donation_repo = Arc::new(PgDonationRepository::new(
        pool.clone(), 
        campaign_totals_cache.clone(),
        user_campaign_donation_cache.clone())); 
    let campaign_repo = Arc::new(PgCampaignRepository::new(pool.clone()));
    let notification_repo = Arc::new(DbNotificationRepository::new(pool.clone()));
    let new_campaign_subs_repo = Arc::new(DbNewCampaignSubscriptionRepository::new(pool.clone()));
    let statistic_service = Arc::new(StatisticService::new(pool.clone()));
    let wallet_repo = Arc::new(PgWalletRepository::new(pool.clone()));
    let transaction_repo = Arc::new(PgTransactionRepository::new(pool.clone()));

    // Design Patterns
    let campaign_factory = Arc::new(CampaignFactory::new());
    let subscriber_service = Arc::new(SubscriberService::new(notification_repo.clone()));
    
    // Services
    let donation_service = DonationService::new(donation_repo, campaign_repo.clone(), wallet_repo.clone());
    let campaign_service = Arc::new(CampaignService::new(campaign_repo, campaign_factory.clone()));
    let notification_service = Arc::new(NotificationService::new(
        notification_repo,
        new_campaign_subs_repo,
        subscriber_service,
    ));
    let wallet_service = WalletService::new(wallet_repo, transaction_repo);

    AppState {
        donation_service,
        campaign_service,
        campaign_factory,
        notification_service,
        statistic_service,
        wallet_service
    }
}

/// Add all managed state to the Rocket instance
pub trait StateManagement {
    fn manage_state(self, state: AppState) -> Self;
}

impl StateManagement for Rocket<Build> {
    fn manage_state(self, state: AppState) -> Self {
        self.manage(state.donation_service)
            .manage(state.campaign_factory)
            .manage(state.campaign_service)
            .manage(state.notification_service)
            .manage(state.wallet_service) // WalletService needs to be Send + Sync + 'static
            .manage(state.statistic_service)
    }
}