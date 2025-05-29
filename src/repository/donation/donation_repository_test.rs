use async_trait::async_trait;
use crate::model::donation::donation::{Donation, NewDonationRequest};
use crate::errors::AppError;

// 1. UPDATE THE TRAIT DEFINITION
#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
    // Add the new methods from PgDonationRepository
    async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError>;
    async fn get_user_total_for_campaign(&self, user_id: i32, campaign_id: i32) -> Result<i64, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::donation::donation::{Donation, NewDonationRequest};
    use crate::repository::donation::donation_repository::{CampaignTotalsCache, UserCampaignDonationCache};
    use std::collections::HashMap;
    use std::sync::{
        atomic::{AtomicI32, Ordering},
        Arc
    };
    use chrono::Utc;

    use crate::errors::AppError;
    use tokio::sync::Mutex; // Use tokio's Mutex for async code

    #[derive(Debug, Clone)]
    struct MockWallet {
        id: i32,
        user_id: i32,
        balance: f64,
    }

    #[derive(Debug, Clone)]
    struct MockCampaign {
        id: i32,
        collected_amount: i64,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct MockTransactionRecord {
        pub wallet_id: i32,
        pub transaction_type: String,
        pub amount: f64,
        pub campaign_id: Option<i32>,
    }

    #[derive(Debug, Default)]
    struct MockDonationRepositoryState {
        donations: HashMap<i32, Donation>,
        next_donation_id: AtomicI32,
        wallets: HashMap<i32, MockWallet>, // Keyed by user_id for easy lookup in mock
        next_wallet_id: AtomicI32,
        campaigns: HashMap<i32, MockCampaign>, // Keyed by campaign_id
        transactions: Vec<MockTransactionRecord>,
    }

    #[derive(Debug, Clone)] // Clone is useful for some test setups if you pass repo around
    pub struct MockDonationRepository {
        state: Arc<Mutex<MockDonationRepositoryState>>, // ** MODIFIED **: Ensure this is tokio::sync::Mutex
        campaign_totals_cache: CampaignTotalsCache,
        user_campaign_donation_cache: UserCampaignDonationCache,
    }

    impl MockDonationRepository {
        // ** MODIFIED **: Constructor now accepts cache dependencies
        pub fn new(
            campaign_totals_cache: CampaignTotalsCache,
            user_campaign_donation_cache: UserCampaignDonationCache,
        ) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockDonationRepositoryState {
                    donations: HashMap::new(),
                    next_donation_id: AtomicI32::new(1),
                    // ++ ADDED ++: Initialize new state fields
                    wallets: HashMap::new(),
                    next_wallet_id: AtomicI32::new(1),
                    campaigns: HashMap::new(),
                    transactions: Vec::new(),
                    // ++ END ADDED ++
                })),
                // ++ ADDED ++: Store injected caches
                campaign_totals_cache,
                user_campaign_donation_cache,
                // ++ END ADDED ++
            }
        }

        // ++ ADDED ++: Helper methods to configure the mock's internal state for tests
        pub async fn add_wallet(&self, user_id: i32, initial_balance: f64) -> i32 {
            let mut state = self.state.lock().await;
            let wallet_id = state.next_wallet_id.fetch_add(1, Ordering::SeqCst);
            let wallet = MockWallet { id: wallet_id, user_id, balance: initial_balance };
            state.wallets.insert(user_id, wallet); // Store by user_id for easy access in create()
            wallet_id
        }

        pub async fn add_campaign(&self, campaign_id: i32, initial_collected_amount: i64) {
            let mut state = self.state.lock().await;
            let campaign = MockCampaign { id: campaign_id, collected_amount: initial_collected_amount };
            state.campaigns.insert(campaign_id, campaign);
        }

        // Helpers to inspect mock state for assertions
        pub async fn get_transactions(&self) -> Vec<MockTransactionRecord> {
            self.state.lock().await.transactions.clone()
        }
        pub async fn get_wallet_balance(&self, user_id: i32) -> Option<f64> {
            self.state.lock().await.wallets.get(&user_id).map(|w| w.balance)
        }
        pub async fn get_campaign_collected_amount(&self, campaign_id: i32) -> Option<i64> {
            self.state.lock().await.campaigns.get(&campaign_id).map(|c| c.collected_amount)
        }
        // ++ END ADDED ++
    }

    #[async_trait]
    impl DonationRepository for MockDonationRepository {
        // ** MODIFIED SIGNIFICANTLY **: This method now simulates wallet, campaign, transaction logic, and cache updates
        async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError> {
            let mut state = self.state.lock().await; // ++ USES ASYNC LOCK ++

            // ++ ADDED: Wallet check and update simulation ++
            let wallet = state.wallets.get_mut(&user_id)
                .ok_or_else(|| AppError::NotFound(format!("Wallet not found for user_id: {}", user_id)))?;
            if wallet.balance < new_donation.amount as f64 {
                return Err(AppError::DatabaseError("Insufficient wallet balance".to_string()));
            }
            let wallet_id_for_tx = wallet.id; // Capture wallet_id for the transaction log
            wallet.balance -= new_donation.amount as f64;
            // ++ END ADDED ++

            // ++ ADDED: Campaign check and update simulation ++
            let campaign = state.campaigns.get_mut(&new_donation.campaign_id)
                .ok_or_else(|| AppError::NotFound(format!("Campaign with id {} not found.", new_donation.campaign_id)))?;
            campaign.collected_amount += new_donation.amount;
            // ++ END ADDED ++

            // Original donation creation logic (mostly similar)
            let donation_id = state.next_donation_id.fetch_add(1, Ordering::SeqCst);
            let donation = Donation {
                id: donation_id,
                user_id,
                campaign_id: new_donation.campaign_id,
                amount: new_donation.amount,
                message: new_donation.message.clone(),
                created_at: Utc::now(),
            };
            state.donations.insert(donation_id, donation.clone());

            // ++ ADDED: Transaction logging simulation ++
            let transaction_record = MockTransactionRecord {
                wallet_id: wallet_id_for_tx,
                transaction_type: "donation".to_string(),
                amount: new_donation.amount as f64,
                campaign_id: Some(new_donation.campaign_id),
            };
            state.transactions.push(transaction_record);
            // ++ END ADDED ++

            // ++ ADDED: Cache Update simulation (mirroring PgDonationRepository) ++
            {
                let mut cache_guard = self.campaign_totals_cache.lock().await;
                *cache_guard.entry(donation.campaign_id).or_insert(0) += donation.amount;
            }
            {
                let mut cache_guard = self.user_campaign_donation_cache.lock().await;
                *cache_guard.entry(donation.user_id).or_default().entry(donation.campaign_id).or_insert(0) += donation.amount;
            }
            // ++ END ADDED ++

            Ok(donation)
        }

        // Methods below are largely the same as the "old" mock, but ensure they use async lock
        async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError> {
            let state = self.state.lock().await; // ++ USES ASYNC LOCK ++
            Ok(state.donations.get(&donation_id).cloned())
        }

        async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError> {
            let state = self.state.lock().await; // ++ USES ASYNC LOCK ++
            Ok(state.donations.values().filter(|d| d.campaign_id == campaign_id).cloned().collect())
        }

        async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError> {
            let state = self.state.lock().await; // ++ USES ASYNC LOCK ++
            Ok(state.donations.values().filter(|d| d.user_id == user_id).cloned().collect())
        }

        async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError> {
            let mut state = self.state.lock().await; // ++ USES ASYNC LOCK ++
            if let Some(donation) = state.donations.get_mut(&donation_id) {
                if donation.user_id == user_id {
                    donation.message = message;
                    Ok(1)
                } else { Ok(0) }
            } else { Ok(0) }
        }

        // ** MODIFIED **: Getter methods now interact with caches, similar to PgDonationRepository
        async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError> {
            // ++ ADDED: Cache read attempt ++
            {
                let cache_guard = self.campaign_totals_cache.lock().await;
                if let Some(total) = cache_guard.get(&campaign_id) {
                    return Ok(*total); // Return cached value if found
                }
            }
            // ++ END ADDED ++

            // If cache miss, calculate from mock state (simulates DB query)
            let state = self.state.lock().await;
            let total_amount = state.donations.values().filter(|d| d.campaign_id == campaign_id).map(|d| d.amount).sum();

            // ++ ADDED: Cache write after calculation ++
            let mut cache_guard = self.campaign_totals_cache.lock().await;
            cache_guard.insert(campaign_id, total_amount);
            // ++ END ADDED ++
            Ok(total_amount)
        }

        async fn get_user_total_for_campaign(&self, user_id: i32, campaign_id: i32) -> Result<i64, AppError> {
            // ++ ADDED: Cache read attempt ++
            {
                let cache_guard = self.user_campaign_donation_cache.lock().await;
                if let Some(user_donations) = cache_guard.get(&user_id) {
                    if let Some(total) = user_donations.get(&campaign_id) {
                        return Ok(*total); // Return cached value
                    }
                }
            }
            // ++ END ADDED ++

            // If cache miss, calculate
            let state = self.state.lock().await;
            let total_amount = state.donations.values().filter(|d| d.user_id == user_id && d.campaign_id == campaign_id).map(|d| d.amount).sum();
            
            // ++ ADDED: Cache write after calculation ++
            let mut cache_guard = self.user_campaign_donation_cache.lock().await;
            cache_guard.entry(user_id).or_default().insert(campaign_id, total_amount);
            // ++ END ADDED ++
            Ok(total_amount)
        }
    }

    // ** MODIFIED **: Helper function to get repository pre-configured with caches for tests.
    // This replaces the simpler `get_repository()` from the old tests.
    fn get_repository_with_caches() -> (MockDonationRepository, CampaignTotalsCache, UserCampaignDonationCache) {
        let campaign_totals_cache = Arc::new(Mutex::new(HashMap::new()));
        let user_campaign_donation_cache = Arc::new(Mutex::new(HashMap::new()));
        let repo = MockDonationRepository::new( // Pass caches to the mock's constructor
            Arc::clone(&campaign_totals_cache),
            Arc::clone(&user_campaign_donation_cache),
        );
        (repo, campaign_totals_cache, user_campaign_donation_cache)
    }

    // --- Existing Tests (mostly unchanged) ---

    #[tokio::test]
    async fn test_create_donation_success() {
        // ** MODIFIED **: Use new helper to get repo and cache handles
        let (repo, campaign_totals_cache, user_campaign_donation_cache) = get_repository_with_caches();

        let user_id = 1;
        let campaign_id = 101;
        let initial_wallet_balance = 100.0;
        let donation_amount = 50;

        // ++ ADDED ++: Setup mock wallet and campaign state before calling create
        let wallet_id = repo.add_wallet(user_id, initial_wallet_balance).await;
        repo.add_campaign(campaign_id, 0).await;
        // ++ END ADDED ++

        let new_donation_req = NewDonationRequest { campaign_id, amount: donation_amount, message: Some("Test donation".to_string()) };
        let result = repo.create(user_id, &new_donation_req).await;
        assert!(result.is_ok(), "Failed to create donation: {:?}", result.err());
        let created_donation = result.unwrap();

        // Assertions for created donation (similar to before but ensure IDs are handled if mock changes)
        assert_eq!(created_donation.user_id, user_id);
        assert_eq!(created_donation.campaign_id, campaign_id);
        assert_eq!(created_donation.amount, donation_amount);

        // ++ ADDED ++: Assertions for side effects: wallet balance, campaign amount, transaction log, caches
        let expected_wallet_balance = initial_wallet_balance - donation_amount as f64;
        assert_eq!(repo.get_wallet_balance(user_id).await, Some(expected_wallet_balance));
        assert_eq!(repo.get_campaign_collected_amount(campaign_id).await, Some(donation_amount));

        let transactions = repo.get_transactions().await;
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0], MockTransactionRecord {
            wallet_id,
            transaction_type: "donation".to_string(),
            amount: donation_amount as f64,
            campaign_id: Some(campaign_id),
        });

        assert_eq!(*campaign_totals_cache.lock().await.get(&campaign_id).unwrap(), donation_amount);
        assert_eq!(*user_campaign_donation_cache.lock().await.get(&user_id).unwrap().get(&campaign_id).unwrap(), donation_amount);
        // ++ END ADDED ++
    }

    #[tokio::test]
    async fn test_create_donation_no_message() {
        // 1. Repository Setup: Get repo and cache handles
        let (repo, campaign_totals_cache, user_campaign_donation_cache) = get_repository_with_caches();

        let user_id = 2;
        let campaign_id = 102;
        let initial_wallet_balance = 100.0; // Example balance, must be >= donation amount
        let donation_amount = 25;

        // 2. State Setup: Add mock wallet and campaign
        let wallet_id = repo.add_wallet(user_id, initial_wallet_balance).await;
        repo.add_campaign(campaign_id, 0).await; // Initial collected amount 0

        // The NewDonationRequest itself is fine as it was
        let new_donation_req = NewDonationRequest {
            campaign_id,
            amount: donation_amount,
            message: None, // This is the key part being tested
        };

        // 3. Call `create`
        let result = repo.create(user_id, &new_donation_req).await;
        assert!(result.is_ok(), "Failed to create donation: {:?}", result.err());
        let created_donation = result.unwrap();

        // 4. Assertions
        // Primary assertion: message is None
        assert_eq!(created_donation.message, None);

        // (Optional but recommended) Assert other properties of the created donation
        assert_eq!(created_donation.user_id, user_id);
        assert_eq!(created_donation.campaign_id, campaign_id);
        assert_eq!(created_donation.amount, donation_amount);
        // The ID is internally generated, so you might not assert its specific value unless
        // your mock guarantees a sequence you can predict (like starting from 1).
        // For this example, we'll skip asserting the exact ID.

        // (Optional but recommended) Assert side effects, similar to test_create_donation_success
        let expected_wallet_balance = initial_wallet_balance - donation_amount as f64;
        assert_eq!(
            repo.get_wallet_balance(user_id).await,
            Some(expected_wallet_balance),
            "Wallet balance was not updated correctly."
        );

        assert_eq!(
            repo.get_campaign_collected_amount(campaign_id).await,
            Some(donation_amount),
            "Campaign collected amount was not updated correctly."
        );

        let transactions = repo.get_transactions().await;
        assert_eq!(transactions.len(), 1, "Transaction was not logged.");
        assert_eq!(
            transactions[0],
            MockTransactionRecord {
                wallet_id,
                transaction_type: "donation".to_string(),
                amount: donation_amount as f64,
                campaign_id: Some(campaign_id),
            },
            "Transaction record is incorrect."
        );

        let campaign_totals_guard = campaign_totals_cache.lock().await;
        assert_eq!(
            campaign_totals_guard.get(&campaign_id),
            Some(&donation_amount),
            "Campaign totals cache not updated correctly."
        );

        let user_campaign_guard = user_campaign_donation_cache.lock().await;
        assert_eq!(
            user_campaign_guard.get(&user_id).and_then(|ud| ud.get(&campaign_id)),
            Some(&donation_amount),
            "User campaign donation cache not updated correctly."
        );
    }

    #[tokio::test]
    async fn test_find_by_id_exists() {
        let (repo, _, _) = get_repository_with_caches();
        let user_id = 5;
        let campaign_id = 105;
        repo.add_wallet(user_id, 100.0).await; // ++ ADDED: Setup state ++
        repo.add_campaign(campaign_id, 0).await; // ++ ADDED: Setup state ++
        let new_donation_req = NewDonationRequest { campaign_id, amount: 75, message: Some("Find me".to_string()) };
        let created_donation = repo.create(user_id, &new_donation_req).await.unwrap();

        let found_donation_opt = repo.find_by_id(created_donation.id).await.unwrap();
        assert!(found_donation_opt.is_some());
        assert_eq!(found_donation_opt.unwrap().id, created_donation.id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_exists() {
        let (repo, _, _) = get_repository_with_caches(); // ** MODIFIED **
        let found_donation_opt = repo.find_by_id(99999).await.unwrap();
        assert!(found_donation_opt.is_none());
    }

     #[tokio::test]
    async fn test_find_by_campaign() {
        let (repo, _, _) = get_repository_with_caches(); // ** MODIFIED **
        let campaign_id_target = 201;
        let campaign_id_other = 202;

        // ++ ADDED: Setup wallets and campaigns for donations to succeed ++
        repo.add_wallet(1, 100.0).await;
        repo.add_wallet(2, 100.0).await;
        repo.add_wallet(3, 100.0).await;
        repo.add_campaign(campaign_id_target, 0).await;
        repo.add_campaign(campaign_id_other, 0).await;
        // ++ END ADDED ++

        repo.create(1, &NewDonationRequest { campaign_id: campaign_id_target, amount: 10, message: None }).await.unwrap();
        repo.create(2, &NewDonationRequest { campaign_id: campaign_id_target, amount: 20, message: None }).await.unwrap();
        repo.create(3, &NewDonationRequest { campaign_id: campaign_id_other, amount: 30, message: None }).await.unwrap();

        let donations = repo.find_by_campaign(campaign_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.campaign_id == campaign_id_target));
    }

    #[tokio::test]
    async fn test_find_by_user() {
        let (repo, _, _) = get_repository_with_caches(); // ** MODIFIED **
        let user_id_target = 51;
        let user_id_other = 52;

        // ++ ADDED: Setup state ++
        repo.add_wallet(user_id_target, 100.0).await;
        repo.add_wallet(user_id_other, 100.0).await;
        repo.add_campaign(301, 0).await;
        repo.add_campaign(302, 0).await;
        repo.add_campaign(303, 0).await;
        // ++ END ADDED ++

        repo.create(user_id_target, &NewDonationRequest { campaign_id: 301, amount: 15, message: None }).await.unwrap();
        repo.create(user_id_target, &NewDonationRequest { campaign_id: 302, amount: 25, message: None }).await.unwrap();
        repo.create(user_id_other, &NewDonationRequest { campaign_id: 303, amount: 35, message: None }).await.unwrap();

        let donations = repo.find_by_user(user_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.user_id == user_id_target));
    }

    #[tokio::test]
    async fn test_update_message_success() {
        let (repo, _, _) = get_repository_with_caches(); // ** MODIFIED **
        let user_id = 61;
        // ++ ADDED: Setup state ++
        repo.add_wallet(user_id, 100.0).await;
        repo.add_campaign(401, 0).await;
        // ++ END ADDED ++
        let initial_donation = repo.create(user_id, &NewDonationRequest { campaign_id: 401, amount: 10, message: Some("Initial".into()) }).await.unwrap();
        
        let new_message = Some("Updated".to_string());
        let rows_affected = repo.update_message(initial_donation.id, user_id, new_message.clone()).await.unwrap();
        assert_eq!(rows_affected, 1);
        assert_eq!(repo.find_by_id(initial_donation.id).await.unwrap().unwrap().message, new_message);
    }

    // ++ ADDED ++: New tests for cache behavior of getter methods
    #[tokio::test]
    async fn test_get_total_donated_for_campaign_cache_behavior() {
        let (repo, campaign_totals_cache, _) = get_repository_with_caches();
        let campaign_id = 501;
        repo.add_wallet(1, 200.0).await; repo.add_campaign(campaign_id, 0).await;

        // Donations will update cache via create()
        repo.create(1, &NewDonationRequest { campaign_id, amount: 100, message: None }).await.unwrap();
        repo.create(1, &NewDonationRequest { campaign_id, amount: 50, message: None }).await.unwrap();
        
        // Verify cache was populated by create()
        assert_eq!(*campaign_totals_cache.lock().await.get(&campaign_id).unwrap(), 150);

        // First call to getter should hit the cache populated by create()
        let total1 = repo.get_total_donated_for_campaign(campaign_id).await.unwrap();
        assert_eq!(total1, 150);

        // Manually clear cache to simulate a scenario where cache is not fresh
        campaign_totals_cache.lock().await.clear();

        // Second call to getter: cache miss, should calculate, populate cache, and return
        let total2 = repo.get_total_donated_for_campaign(campaign_id).await.unwrap();
        assert_eq!(total2, 150);
        assert_eq!(*campaign_totals_cache.lock().await.get(&campaign_id).unwrap(), 150); // Cache repopulated

        // Third call to getter: should be a cache hit now
        let total3 = repo.get_total_donated_for_campaign(campaign_id).await.unwrap();
        assert_eq!(total3, 150);
    }

    #[tokio::test]
    async fn test_get_user_total_for_campaign_cache_behavior() {
        let (repo, _, user_campaign_donation_cache) = get_repository_with_caches();
        let user_id = 71; let campaign_id = 601;
        repo.add_wallet(user_id, 200.0).await; repo.add_campaign(campaign_id, 0).await;

        repo.create(user_id, &NewDonationRequest { campaign_id, amount: 25, message: None }).await.unwrap();
        repo.create(user_id, &NewDonationRequest { campaign_id, amount: 75, message: None }).await.unwrap();
        
        assert_eq!(*user_campaign_donation_cache.lock().await.get(&user_id).unwrap().get(&campaign_id).unwrap(), 100);

        let total1 = repo.get_user_total_for_campaign(user_id, campaign_id).await.unwrap();
        assert_eq!(total1, 100); // Cache hit

        user_campaign_donation_cache.lock().await.clear(); // Manually clear

        let total2 = repo.get_user_total_for_campaign(user_id, campaign_id).await.unwrap();
        assert_eq!(total2, 100); // Cache miss, then populate
        assert_eq!(*user_campaign_donation_cache.lock().await.get(&user_id).unwrap().get(&campaign_id).unwrap(), 100);

        let total3 = repo.get_user_total_for_campaign(user_id, campaign_id).await.unwrap();
        assert_eq!(total3, 100); // Cache hit
    }
    // ++ END ADDED ++

        #[tokio::test]
    async fn test_update_message_to_none() {
        // Use new helper to get repo (caches not directly asserted here, but repo uses them)
        let (repo, _, _) = get_repository_with_caches();
        let user_id = 62;
        let campaign_id = 402;
        let donation_amount = 110;

        // Setup mock wallet and campaign for the initial donation
        repo.add_wallet(user_id, donation_amount as f64 + 10.0).await; // Ensure enough balance
        repo.add_campaign(campaign_id, 0).await;

        // Create the initial donation
        let initial_donation = repo.create(user_id, &NewDonationRequest {
            campaign_id,
            amount: donation_amount,
            message: Some("A message to clear".to_string()),
        }).await.unwrap();

        // Update the message to None
        let rows_affected = repo.update_message(initial_donation.id, user_id, None).await.unwrap();
        assert_eq!(rows_affected, 1);

        // Verify the message is None
        let updated_donation = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(updated_donation.message, None);
    }

    #[tokio::test]
    async fn test_update_message_donation_not_found() {
        // Use new helper
        let (repo, _, _) = get_repository_with_caches();
        let user_id = 63; // This user ID doesn't matter much as the donation ID won't exist
        let non_existent_donation_id = 9999;
        let new_message = Some("This won't be set".to_string());

        // No need to create a donation here, as we're testing the "not found" case for update_message
        let rows_affected = repo.update_message(non_existent_donation_id, user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0);
    }

    #[tokio::test]
    async fn test_update_message_user_mismatch() {
        // Use new helper
        let (repo, _, _) = get_repository_with_caches();
        let owner_user_id = 64;
        let other_user_id = 65; // The user attempting the unauthorized update
        let campaign_id = 404;
        let donation_amount = 120;

        // Setup mock wallet and campaign for the original owner's donation
        repo.add_wallet(owner_user_id, donation_amount as f64 + 10.0).await;
        // We don't necessarily need a wallet for other_user_id for this specific test,
        // as the update should fail before any wallet interaction for `update_message`.
        repo.add_campaign(campaign_id, 0).await;

        // Create the initial donation by the owner
        let initial_donation = repo.create(owner_user_id, &NewDonationRequest {
            campaign_id,
            amount: donation_amount,
            message: Some("Original message by owner".to_string()),
        }).await.unwrap();

        let new_message = Some("Attempted update by other user".to_string());

        // Attempt to update the message as `other_user_id`
        let rows_affected = repo.update_message(initial_donation.id, other_user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0, "Update should fail if user_id does not match");

        // Verify the original message remains unchanged
        let donation_after_attempt = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(donation_after_attempt.message, Some("Original message by owner".to_string()));
    }

    // 3. ADD NEW TESTS FOR THE NEW METHODS
    #[tokio::test]
    async fn test_get_total_donated_for_campaign_multiple_donations() {
        let (repo, _, _) = get_repository_with_caches(); // ** MODIFIED **
        let campaign_id = 501;
        // ++ ADDED: Setup state ++
        repo.add_wallet(1, 300.0).await; repo.add_wallet(2, 100.0).await;
        repo.add_campaign(campaign_id, 0).await; repo.add_campaign(502, 0).await;

        repo.create(1, &NewDonationRequest { campaign_id, amount: 100, message: None }).await.unwrap();
        repo.create(2, &NewDonationRequest { campaign_id, amount: 50, message: None }).await.unwrap(); // Different user
        repo.create(1, &NewDonationRequest { campaign_id: 502, amount: 200, message: None }).await.unwrap(); // Different campaign
        assert_eq!(repo.get_total_donated_for_campaign(campaign_id).await.unwrap(), 150);
    }

    #[tokio::test]
    async fn test_get_total_donated_for_campaign_no_donations() {
        let (repo, _, _) = get_repository_with_caches(); // ** MODIFIED **
        repo.add_campaign(503, 0).await; // Campaign exists but no donations
        assert_eq!(repo.get_total_donated_for_campaign(503).await.unwrap(), 0);
        assert_eq!(repo.get_total_donated_for_campaign(9999).await.unwrap(), 0); // Unknown campaign
    }

    #[tokio::test]
    async fn test_get_total_donated_for_campaign_single_donation() {
        // Use new helper to get repo and cache handles (caches not strictly needed for assertion here, but good practice)
        let (repo, _, _) = get_repository_with_caches();
        let campaign_id = 504;
        let user_id = 1; // Need a user to make the donation
        let donation_amount = 75;

        // Setup mock wallet and campaign
        repo.add_wallet(user_id, donation_amount as f64 + 10.0).await; // Ensure enough balance
        repo.add_campaign(campaign_id, 0).await;

        // Make the donation
        repo.create(user_id, &NewDonationRequest { campaign_id, amount: donation_amount, message: None }).await.unwrap();

        // Assert the total
        let total = repo.get_total_donated_for_campaign(campaign_id).await.unwrap();
        assert_eq!(total, donation_amount);
    }

    #[tokio::test]
    async fn test_get_user_total_for_campaign_multiple_donations_by_user() {
        let (repo, _, _) = get_repository_with_caches();
        let user_id_target = 71;
        let user_id_other = 72;
        let campaign_id_target = 601;
        let campaign_id_other = 602;

        // Setup mock wallets
        repo.add_wallet(user_id_target, 500.0).await; // Enough for all their donations
        repo.add_wallet(user_id_other, 100.0).await; // Enough for their donation

        // Setup mock campaigns
        repo.add_campaign(campaign_id_target, 0).await;
        repo.add_campaign(campaign_id_other, 0).await;

        // Make donations
        // User 71 donates to target campaign
        repo.create(user_id_target, &NewDonationRequest { campaign_id: campaign_id_target, amount: 25, message: None }).await.unwrap();
        repo.create(user_id_target, &NewDonationRequest { campaign_id: campaign_id_target, amount: 75, message: None }).await.unwrap();
        // User 71 donates to another campaign (should not be counted)
        repo.create(user_id_target, &NewDonationRequest { campaign_id: campaign_id_other, amount: 100, message: None }).await.unwrap();
        // User 72 donates to target campaign (should not be counted for user_id_target)
        repo.create(user_id_other, &NewDonationRequest { campaign_id: campaign_id_target, amount: 50, message: None }).await.unwrap();

        // Assert total for user_id_target on campaign_id_target
        let total = repo.get_user_total_for_campaign(user_id_target, campaign_id_target).await.unwrap();
        assert_eq!(total, 100); // 25 + 75
    }

    #[tokio::test]
    async fn test_get_user_total_for_campaign_no_donations_by_user_for_campaign() {
        let (repo, _, _) = get_repository_with_caches();
        let user_id_target = 73; // User we are checking
        let user_id_other_donor = 74;
        let campaign_id_target = 603; // Campaign we are checking for user_id_target
        let campaign_id_user_target_donates_to = 604;

        // Setup wallets
        repo.add_wallet(user_id_target, 100.0).await;
        repo.add_wallet(user_id_other_donor, 100.0).await;

        // Setup campaigns
        repo.add_campaign(campaign_id_target, 0).await;
        repo.add_campaign(campaign_id_user_target_donates_to, 0).await;

        // User 73 donates to a *different* campaign
        repo.create(user_id_target, &NewDonationRequest { campaign_id: campaign_id_user_target_donates_to, amount: 30, message: None }).await.unwrap();
        // Another user (74) donates to the campaign we are interested in (603)
        repo.create(user_id_other_donor, &NewDonationRequest { campaign_id: campaign_id_target, amount: 60, message: None }).await.unwrap();

        // Assert total for user_id_target on campaign_id_target should be 0
        let total = repo.get_user_total_for_campaign(user_id_target, campaign_id_target).await.unwrap();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_get_user_total_for_campaign_no_donations_at_all() {
        let (repo, _, _) = get_repository_with_caches();
        let user_id = 75;
        let campaign_id = 605;

        // We don't even need to add wallets or campaigns if there are no donations.
        // The mock's `create` would fail, but `get_user_total_for_campaign` should
        // gracefully return 0 if no donations match.
        // However, for consistency and to test the path where the campaign might exist but has no donations from this user:
        repo.add_campaign(campaign_id, 0).await; // Campaign exists
        // repo.add_wallet(user_id, 10.0).await; // User might exist but hasn't donated to this campaign

        let total = repo.get_user_total_for_campaign(user_id, campaign_id).await.unwrap();
        assert_eq!(total, 0);

        // Test for a user and campaign that are completely unknown to the donation records
        let total_unknown_user_campaign = repo.get_user_total_for_campaign(999, 9999).await.unwrap();
        assert_eq!(total_unknown_user_campaign, 0);
    }

    #[tokio::test]
    async fn test_get_user_total_for_campaign_single_donation_by_user() {
        let (repo, _, _) = get_repository_with_caches();
        let user_id = 76;
        let campaign_id = 606;
        let donation_amount = 90;

        // Setup wallet and campaign
        repo.add_wallet(user_id, donation_amount as f64 + 10.0).await;
        repo.add_campaign(campaign_id, 0).await;

        // Make the donation
        repo.create(user_id, &NewDonationRequest { campaign_id, amount: donation_amount, message: None }).await.unwrap();

        // Assert total for this user and campaign
        let total = repo.get_user_total_for_campaign(user_id, campaign_id).await.unwrap();
        assert_eq!(total, donation_amount);
    }
}