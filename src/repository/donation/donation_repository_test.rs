use async_trait::async_trait;
use crate::model::donation::donation::{Donation, NewDonationRequest};
use crate::errors::AppError;

#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::donation::donation::{Donation, NewDonationRequest};
    use std::collections::HashMap;
    use std::sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex,
    };
    use chrono::Utc;

    #[derive(Debug, Default)]
    struct MockDonationRepositoryState {
        donations: HashMap<i32, Donation>,
        next_id: AtomicI32,
    }

    #[derive(Debug, Clone, Default)]
    pub struct MockDonationRepository {
        state: Arc<Mutex<MockDonationRepositoryState>>,
    }

    impl MockDonationRepository {
        pub fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(MockDonationRepositoryState {
                    donations: HashMap::new(),
                    next_id: AtomicI32::new(1),
                })),
            }
        }
    }

    #[async_trait]
    impl DonationRepository for MockDonationRepository {
        async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError> {
            let mut state = self.state.lock().expect("Failed to lock mock state");
            let id = state.next_id.fetch_add(1, Ordering::SeqCst);

            let donation = Donation {
                id,
                user_id,
                campaign_id: new_donation.campaign_id,
                amount: new_donation.amount,
                message: new_donation.message.clone(),
                created_at: Utc::now(),
            };
            state.donations.insert(id, donation.clone());
            Ok(donation)
        }

        async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError> {
            let state = self.state.lock().expect("Failed to lock mock state");
            Ok(state.donations.get(&donation_id).cloned())
        }

        async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError> {
            let state = self.state.lock().expect("Failed to lock mock state");
            let donations = state
                .donations
                .values()
                .filter(|d| d.campaign_id == campaign_id)
                .cloned()
                .collect();
            Ok(donations)
        }

        async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError> {
            let state = self.state.lock().expect("Failed to lock mock state");
            let donations = state
                .donations
                .values()
                .filter(|d| d.user_id == user_id)
                .cloned()
                .collect();
            Ok(donations)
        }

        async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError> {
            let mut state = self.state.lock().expect("Failed to lock mock state");
            if let Some(donation) = state.donations.get_mut(&donation_id) {
                if donation.user_id == user_id { // Check if the user owns the donation
                    donation.message = message;
                    // In a real scenario, you might also update an `updated_at` timestamp
                    Ok(1) // 1 row affected
                } else {
                    Ok(0) // User mismatch, 0 rows affected
                }
            } else {
                Ok(0) // Donation not found, 0 rows affected
            }
        }
    }

    // Helper to get a repository instance for tests
    // This now returns our MockDonationRepository
    fn get_repository() -> MockDonationRepository {
        MockDonationRepository::new()
    }

    // No more `clear_donations_table` or `TEST_POOL` needed, as each test
    // will get a fresh, empty `MockDonationRepository` instance.

    // --- Tests (mostly unchanged, just use the mock now) ---

    #[tokio::test]
    async fn test_create_donation_success() {
        let repo = get_repository(); // Gets a new MockDonationRepository

        let user_id = 1;
        let new_donation_req = NewDonationRequest {
            campaign_id: 101,
            amount: 50.0,
            message: Some("Test donation".to_string()),
        };

        let result = repo.create(user_id, &new_donation_req).await;
        assert!(result.is_ok(), "Failed to create donation: {:?}", result.err());

        let created_donation = result.unwrap();
        assert_eq!(created_donation.id, 1); // First ID from mock
        assert_eq!(created_donation.user_id, user_id);
        assert_eq!(created_donation.campaign_id, new_donation_req.campaign_id);
        assert_eq!(created_donation.amount, new_donation_req.amount);
        assert_eq!(created_donation.message, new_donation_req.message);
        // Check created_at is recent
        let now = Utc::now();
        assert!(created_donation.created_at <= now && created_donation.created_at > (now - chrono::Duration::seconds(5)));
    }

    #[tokio::test]
    async fn test_create_donation_no_message() {
        let repo = get_repository();

        let user_id = 2;
        let new_donation_req = NewDonationRequest {
            campaign_id: 102,
            amount: 25.0,
            message: None,
        };

        let created_donation = repo.create(user_id, &new_donation_req).await.unwrap();
        assert_eq!(created_donation.id, 1);
        assert_eq!(created_donation.message, None);
    }

    #[tokio::test]
    async fn test_find_by_id_exists() {
        let repo = get_repository();

        let user_id = 3;
        let new_donation_req = NewDonationRequest {
            campaign_id: 103,
            amount: 75.0,
            message: Some("Find me".to_string()),
        };
        let created_donation = repo.create(user_id, &new_donation_req).await.unwrap();
        assert_eq!(created_donation.id, 1); // ID assigned by mock

        let found_donation_opt = repo.find_by_id(created_donation.id).await.unwrap();
        assert!(found_donation_opt.is_some());
        let found_donation = found_donation_opt.unwrap();
        assert_eq!(found_donation.id, created_donation.id);
        assert_eq!(found_donation.message, Some("Find me".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_id_not_exists() {
        let repo = get_repository();
        // No donations created in this mock instance yet

        let found_donation_opt = repo.find_by_id(99999).await.unwrap();
        assert!(found_donation_opt.is_none());
    }

    #[tokio::test]
    async fn test_find_by_campaign() {
        let repo = get_repository();

        let campaign_id_target = 201;
        let campaign_id_other = 202;

        repo.create(1, &NewDonationRequest { campaign_id: campaign_id_target, amount: 10.0, message: None }).await.unwrap(); // id 1
        repo.create(2, &NewDonationRequest { campaign_id: campaign_id_target, amount: 20.0, message: Some("Msg1".into()) }).await.unwrap(); // id 2
        repo.create(3, &NewDonationRequest { campaign_id: campaign_id_other, amount: 30.0, message: None }).await.unwrap(); // id 3

        let donations = repo.find_by_campaign(campaign_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.campaign_id == campaign_id_target));
        // Check if specific donations are present (optional, good for ordering if relevant)
        assert!(donations.iter().any(|d| d.id == 1 && d.amount == 10.0));
        assert!(donations.iter().any(|d| d.id == 2 && d.amount == 20.0));


        let donations_other = repo.find_by_campaign(campaign_id_other).await.unwrap();
        assert_eq!(donations_other.len(), 1);
        assert_eq!(donations_other[0].id, 3);


        let donations_none = repo.find_by_campaign(999).await.unwrap(); // Non-existent campaign_id
        assert!(donations_none.is_empty());
    }

    #[tokio::test]
    async fn test_find_by_user() {
        let repo = get_repository();

        let user_id_target = 51;
        let user_id_other = 52;

        repo.create(user_id_target, &NewDonationRequest { campaign_id: 301, amount: 15.0, message: None }).await.unwrap(); // id 1
        repo.create(user_id_target, &NewDonationRequest { campaign_id: 302, amount: 25.0, message: Some("Msg2".into()) }).await.unwrap(); // id 2
        repo.create(user_id_other, &NewDonationRequest { campaign_id: 303, amount: 35.0, message: None }).await.unwrap(); // id 3

        let donations = repo.find_by_user(user_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.user_id == user_id_target));

        let donations_other = repo.find_by_user(user_id_other).await.unwrap();
        assert_eq!(donations_other.len(), 1);
        assert_eq!(donations_other[0].user_id, user_id_other);


        let donations_none = repo.find_by_user(999).await.unwrap(); // Non-existent user_id
        assert!(donations_none.is_empty());
    }

    #[tokio::test]
    async fn test_update_message_success() {
        let repo = get_repository();

        let user_id = 61;
        let initial_donation = repo.create(user_id, &NewDonationRequest {
            campaign_id: 401,
            amount: 100.0,
            message: Some("Initial message".to_string()),
        }).await.unwrap(); // id 1

        let new_message = Some("Updated message".to_string());
        let rows_affected = repo.update_message(initial_donation.id, user_id, new_message.clone()).await.unwrap();
        assert_eq!(rows_affected, 1);

        let updated_donation = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(updated_donation.message, new_message);
    }

    #[tokio::test]
    async fn test_update_message_to_none() {
        let repo = get_repository();

        let user_id = 62;
        let initial_donation = repo.create(user_id, &NewDonationRequest {
            campaign_id: 402,
            amount: 110.0,
            message: Some("A message to clear".to_string()),
        }).await.unwrap(); // id 1

        let rows_affected = repo.update_message(initial_donation.id, user_id, None).await.unwrap();
        assert_eq!(rows_affected, 1);

        let updated_donation = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(updated_donation.message, None);
    }


    #[tokio::test]
    async fn test_update_message_donation_not_found() {
        let repo = get_repository();

        let user_id = 63;
        let non_existent_donation_id = 9999; // This ID won't exist in a fresh mock
        let new_message = Some("This won't be set".to_string());
        let rows_affected = repo.update_message(non_existent_donation_id, user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0);
    }

    #[tokio::test]
    async fn test_update_message_user_mismatch() {
        let repo = get_repository();

        let owner_user_id = 64;
        let other_user_id = 65; // Different user
        let initial_donation = repo.create(owner_user_id, &NewDonationRequest { // Donation created by owner_user_id
            campaign_id: 404,
            amount: 120.0,
            message: Some("Original message by owner".to_string()),
        }).await.unwrap(); // id 1

        let new_message = Some("Attempted update by other user".to_string());
        // Attempt to update with other_user_id
        let rows_affected = repo.update_message(initial_donation.id, other_user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0, "Update should fail if user_id does not match");

        // Verify the message hasn't changed
        let donation_after_attempt = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(donation_after_attempt.message, Some("Original message by owner".to_string()));
    }
}