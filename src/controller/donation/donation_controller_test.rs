#[cfg(test)]
mod tests {
    // To import controller functions and types from parent module
    use crate::model::donation::donation::{Donation, NewDonationRequest};
    use crate::service::donation::donation_service::DonationService;
    use crate::errors::AppError;
    use crate::controller::donation::donation_controller::routes;
    // use crate::auth::auth::AuthUser; // Still using DummyAuthUser from controller

    use rocket::local::blocking::Client;
    use rocket::http::{Status, ContentType};
    use rocket::serde::json::json;
    use std::sync::{Arc, Mutex};
    use chrono::{Utc, Duration};
    use async_trait::async_trait;

    // Import repository traits and dependent models
    use crate::repository::donation::donation_repository::DonationRepository;
    use crate::repository::campaign::campaign_repository::CampaignRepository;
    use crate::model::campaign::campaign::{Campaign, CampaignStatus}; // Import CampaignStatus as well

    // --- Manual Mock for DonationRepository (remains the same as you provided) ---
    #[derive(Clone)]
    struct MockDonationRepository {
        create_fn: Arc<Mutex<Option<Box<dyn Fn(i32, NewDonationRequest) -> Result<Donation, AppError> + Send + Sync>>>>,
        find_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i32) -> Result<Option<Donation>, AppError> + Send + Sync>>>>,
        find_by_campaign_fn: Arc<Mutex<Option<Box<dyn Fn(i32) -> Result<Vec<Donation>, AppError> + Send + Sync>>>>,
        find_by_user_fn: Arc<Mutex<Option<Box<dyn Fn(i32) -> Result<Vec<Donation>, AppError> + Send + Sync>>>>,
        update_message_fn: Arc<Mutex<Option<Box<dyn Fn(i32, i32, Option<String>) -> Result<u64, AppError> + Send + Sync>>>>,
    }

    impl MockDonationRepository {
        fn new() -> Self {
            MockDonationRepository {
                create_fn: Arc::new(Mutex::new(None)),
                find_by_id_fn: Arc::new(Mutex::new(None)),
                find_by_campaign_fn: Arc::new(Mutex::new(None)),
                find_by_user_fn: Arc::new(Mutex::new(None)),
                update_message_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create(&mut self, f: impl Fn(i32, NewDonationRequest) -> Result<Donation, AppError> + Send + Sync + 'static) {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_find_by_id(&mut self, f: impl Fn(i32) -> Result<Option<Donation>, AppError> + Send + Sync + 'static) {
            *self.find_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_find_by_campaign(&mut self, f: impl Fn(i32) -> Result<Vec<Donation>, AppError> + Send + Sync + 'static) {
            *self.find_by_campaign_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_find_by_user(&mut self, f: impl Fn(i32) -> Result<Vec<Donation>, AppError> + Send + Sync + 'static) {
            *self.find_by_user_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_update_message(&mut self, f: impl Fn(i32, i32, Option<String>) -> Result<u64, AppError> + Send + Sync + 'static) {
            *self.update_message_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl DonationRepository for MockDonationRepository {
        async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError> {
            let guard = self.create_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id, new_donation.clone())
            } else {
                panic!("MockDonationRepository: expect_create was not called or not set");
            }
        }
        async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError> {
            let guard = self.find_by_id_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(donation_id)
            } else {
                panic!("MockDonationRepository: expect_find_by_id was not called or not set");
            }
        }
        async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError> {
            let guard = self.find_by_campaign_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(campaign_id)
            } else {
                panic!("MockDonationRepository: expect_find_by_campaign was not called or not set");
            }
        }
        async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError> {
            let guard = self.find_by_user_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id)
            } else {
                panic!("MockDonationRepository: expect_find_by_user was not called or not set");
            }
        }
        async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError> {
            let guard = self.update_message_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(donation_id, user_id, message.clone())
            } else {
                panic!("MockDonationRepository: expect_update_message was not called or not set");
            }
        }
    }

    // --- Completed Manual Mock for CampaignRepository ---
    #[derive(Clone)]
    struct MockCampaignRepository {
        create_campaign_fn: Arc<Mutex<Option<Box<dyn Fn(Campaign) -> Result<Campaign, AppError> + Send + Sync>>>>,
        get_campaign_fn: Arc<Mutex<Option<Box<dyn Fn(i32) -> Result<Option<Campaign>, AppError> + Send + Sync>>>>,
        update_campaign_fn: Arc<Mutex<Option<Box<dyn Fn(Campaign) -> Result<Campaign, AppError> + Send + Sync>>>>,
        update_campaign_status_fn: Arc<Mutex<Option<Box<dyn Fn(i32, CampaignStatus) -> Result<bool, AppError> + Send + Sync>>>>,
        get_campaigns_by_user_fn: Arc<Mutex<Option<Box<dyn Fn(i32) -> Result<Vec<Campaign>, AppError> + Send + Sync>>>>,
        get_campaigns_by_status_fn: Arc<Mutex<Option<Box<dyn Fn(CampaignStatus) -> Result<Vec<Campaign>, AppError> + Send + Sync>>>>,
        get_all_campaigns_fn: Arc<Mutex<Option<Box<dyn Fn() -> Result<Vec<Campaign>, AppError> + Send + Sync + 'static>>>>,
        delete_campaign_fn: Arc<Mutex<Option<Box<dyn Fn(i32) -> Result<bool, AppError> + Send + Sync + 'static>>>>,
    }

    impl MockCampaignRepository {
        fn new() -> Self {
            MockCampaignRepository {
                create_campaign_fn: Arc::new(Mutex::new(None)),
                get_campaign_fn: Arc::new(Mutex::new(None)),
                update_campaign_fn: Arc::new(Mutex::new(None)),
                update_campaign_status_fn: Arc::new(Mutex::new(None)),
                get_campaigns_by_user_fn: Arc::new(Mutex::new(None)),
                get_campaigns_by_status_fn: Arc::new(Mutex::new(None)),
                get_all_campaigns_fn: Arc::new(Mutex::new(None)),
                delete_campaign_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create_campaign(&mut self, f: impl Fn(Campaign) -> Result<Campaign, AppError> + Send + Sync + 'static) {
            *self.create_campaign_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_get_campaign(&mut self, f: impl Fn(i32) -> Result<Option<Campaign>, AppError> + Send + Sync + 'static) {
            *self.get_campaign_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_update_campaign(&mut self, f: impl Fn(Campaign) -> Result<Campaign, AppError> + Send + Sync + 'static) {
            *self.update_campaign_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_update_campaign_status(&mut self, f: impl Fn(i32, CampaignStatus) -> Result<bool, AppError> + Send + Sync + 'static) {
            *self.update_campaign_status_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_get_campaigns_by_user(&mut self, f: impl Fn(i32) -> Result<Vec<Campaign>, AppError> + Send + Sync + 'static) {
            *self.get_campaigns_by_user_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_get_campaigns_by_status(&mut self, f: impl Fn(CampaignStatus) -> Result<Vec<Campaign>, AppError> + Send + Sync + 'static) {
            *self.get_campaigns_by_status_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_get_all_campaigns(&mut self, f: impl Fn() -> Result<Vec<Campaign>, AppError> + Send + Sync + 'static) {
            *self.get_all_campaigns_fn.lock().unwrap() = Some(Box::new(f));
        }
        fn expect_delete_campaign(&mut self, f: impl Fn(i32) -> Result<bool, AppError> + Send + Sync + 'static) {
            *self.delete_campaign_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl CampaignRepository for MockCampaignRepository {
        async fn create_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError> {
            let guard = self.create_campaign_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(campaign) // Campaign is Clone, so direct pass is fine if f expects ownership
            } else {
                panic!("MockCampaignRepository: expect_create_campaign was not called or not set");
            }
        }

        async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError> {
            let guard = self.get_campaign_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                let result = f(id);
                println!("[MockCampaignRepository] get_campaign returning: {:?}", result);
                result
            } else {
                panic!("MockCampaignRepository: expect_get_campaign was not called or not set for id: {}", id);
            }
        }

        async fn update_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError> {
            let guard = self.update_campaign_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(campaign)
            } else {
                panic!("MockCampaignRepository: expect_update_campaign was not called or not set");
            }
        }

        async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError> {
            let guard = self.update_campaign_status_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(id, status) // CampaignStatus is Clone
            } else {
                panic!("MockCampaignRepository: expect_update_campaign_status was not called or not set for id: {}", id);
            }
        }

        async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError> {
            let guard = self.get_campaigns_by_user_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id)
            } else {
                panic!("MockCampaignRepository: expect_get_campaigns_by_user was not called or not set for user_id: {}", user_id);
            }
        }

        async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError> {
            let guard = self.get_campaigns_by_status_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(status) // CampaignStatus is Clone
            } else {
                panic!("MockCampaignRepository: expect_get_campaigns_by_status was not called or not set for status: {:?}", status);
            }
        }

        async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError> {
            let f = self.get_all_campaigns_fn.lock().unwrap();
            if let Some(f) = &*f {
                (f)()
            } else {
                panic!("MockCampaignRepository::get_all_campaigns was called without an expectation");
            }
        }

        async fn delete_campaign(&self, id: i32) -> Result<bool, AppError> {
            let f = self.delete_campaign_fn.lock().unwrap();
            if let Some(f) = &*f {
                (f)(id)
            } else {
                panic!("MockCampaignRepository::delete_campaign was called without an expectation");
            }
        }
    }

    // Helper to build a Rocket instance for testing with a DonationService using mocked repositories
    fn create_test_rocket_with_service(service: DonationService) -> rocket::Rocket<rocket::Build> {
        rocket::build()
            .mount("/api", routes()) // Assuming your routes are mounted under /api (routes() from parent module)
            .manage(service) // Manage the DonationService instance directly
    }

    // Helper to create a sample donation for expected results
    fn sample_donation(id: i32, user_id: i32, campaign_id: i32, amount: i64) -> Donation {
        Donation {
            id,
            user_id,
            campaign_id,
            amount,
            message: Some("Test donation".to_string()),
            created_at: Utc::now(),
        }
    }
    
    // Helper to create a sample campaign for tests
    fn sample_campaign(id: i32, user_id: i32, name: &str, status: CampaignStatus) -> Campaign {
        let now = Utc::now();
        Campaign {
            id,
            user_id,
            name: name.to_string(),
            description: "Sample campaign description".to_string(),
            target_amount: 1000,
            collected_amount: 0,
            start_date: now, // Added
            end_date: now + Duration::days(30), // Added - example: 30 days from now
            image_url: Some("http://example.com/default_campaign_image.png".to_string()),
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        }
    }

    // The TestCampaign struct is no longer needed as we use the actual Campaign struct.
    // // pub struct TestCampaign { ... } 

    #[test]
    fn test_make_donation_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mut mock_campaign_repo = MockCampaignRepository::new();

        let expected_user_id_from_controller = 123; // Matches DUMMY_USER_ID in controller
        let expected_campaign_id = 1;
        let expected_amount = 50;
        let returned_donation = sample_donation(1, expected_user_id_from_controller, expected_campaign_id, expected_amount);
        
        // 1. Expect campaign_repo.get_campaign to be called by the service
        // Use the actual Campaign struct and the sample_campaign helper
        let campaign_to_return = sample_campaign(expected_campaign_id, 10, "Active Campaign", CampaignStatus::Active);

        mock_campaign_repo.expect_get_campaign({
            let c = campaign_to_return.clone();
            move |id| {
                assert_eq!(id, expected_campaign_id, "Campaign ID mismatch in get_campaign mock");
                Ok(Some(c.clone()))
            }
        });

        // 2. Expect donation_repo.create to be called by the service
        mock_donation_repo.expect_create({
            let rd = returned_donation.clone();
            move |user_id, req| {
                assert_eq!(user_id, expected_user_id_from_controller, "User ID mismatch in create_donation mock");
                assert_eq!(req.campaign_id, expected_campaign_id, "Campaign ID mismatch in create_donation mock");
                assert_eq!(req.amount, expected_amount, "Amount mismatch in create_donation mock");
                assert_eq!(req.message, Some("Hope this helps!".to_string()), "Message mismatch in create_donation mock");
                Ok(rd.clone())
            }
        });

        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");
        
        let response = client
            .post("/api/donations")
            .header(ContentType::JSON)
            .json(&json!({
                "campaign_id": expected_campaign_id,
                "amount": expected_amount,
                "message": "Hope this helps!"
            }))
            .dispatch();

        assert_eq!(response.status(), Status::Created);
        assert!(response.headers().get_one("Location").is_some());
        let body = response.into_json::<Donation>().unwrap();
        assert_eq!(body.id, returned_donation.id);
        assert_eq!(body.amount, returned_donation.amount);
    }

    #[test]
    fn test_make_donation_campaign_not_found() {
        let mock_donation_repo = MockDonationRepository::new(); // No donation repo calls if campaign not found
        let mut mock_campaign_repo = MockCampaignRepository::new();

        let non_existent_campaign_id = 999;

        mock_campaign_repo.expect_get_campaign(move |id| {
            assert_eq!(id, non_existent_campaign_id);
            Ok(None) // Simulate campaign not found
        });

        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");

        let response = client
            .post("/api/donations")
            .header(ContentType::JSON)
            .json(&json!({
                "campaign_id": non_existent_campaign_id,
                "amount": 50,
                "message": "For a ghost campaign"
            }))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound); // AppError::NotFound maps to 404
        let body = response.into_json::<serde_json::Value>().unwrap();
        assert_eq!(body["error"], "Campaign not found");
    }


    #[test]
    fn test_make_donation_service_validation_error() {
        let mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
        // No expectations on repos are set, as service should validate before calling them.

        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");
        
        let response = client
            .post("/api/donations")
            .header(ContentType::JSON)
            .json(&json!({ "campaign_id": 1, "amount": 0, "message": "Too small" }))
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        let body = response.into_json::<serde_json::Value>().unwrap();
        assert_eq!(body["error"], "Donation amount must be positive");
    }

    // --- Adapting other tests ---
    // (You'll continue to adapt the rest of your tests here, using the mock repositories)

    #[test]
    fn test_delete_donation_message_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new(); // Not used in this service method

        let donation_id_to_delete = 1;
        let user_id_performing_delete = 123; // DUMMY_USER_ID from controller

        mock_donation_repo.expect_update_message(move |d_id, u_id, msg| {
            assert_eq!(d_id, donation_id_to_delete);
            assert_eq!(u_id, user_id_performing_delete);
            assert_eq!(msg, None);
            Ok(1) // 1 row affected
        });

        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");

        let response = client
            .delete(format!("/api/donations/{}/message", donation_id_to_delete))
            // .header(Header::new("X-User-Id", user_id_performing_delete.to_string())) // Header not used by DummyAuthUser
            .dispatch();

        assert_eq!(response.status(), Status::NoContent);
    }

    #[test]
    fn test_delete_donation_message_not_found_after_update() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();

        let donation_id_non_existent = 99;
        let user_id_performing_delete = 123;

        mock_donation_repo.expect_update_message(move |d_id, u_id, msg| {
            assert_eq!(d_id, donation_id_non_existent);
            assert_eq!(u_id, user_id_performing_delete);
            assert_eq!(msg, None);
            Ok(0) // 0 rows affected
        });
        mock_donation_repo.expect_find_by_id(move |d_id| {
            assert_eq!(d_id, donation_id_non_existent);
            Ok(None) // Donation not found by find_by_id
        });


        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");

        let response = client
            .delete(format!("/api/donations/{}/message", donation_id_non_existent))
            .dispatch();
        
        assert_eq!(response.status(), Status::NotFound);
        let body = response.into_json::<serde_json::Value>().unwrap();
        assert_eq!(body["error"], "Donation not found");
    }

    #[test]
    fn test_delete_donation_message_forbidden_after_update() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();

        let donation_id_exists = 1;
        let user_id_performing_delete = 123; // This is DUMMY_USER_ID from controller
        // The service's update_message implies ownership check is in the repo or DB layer.
        // Here, the service logic for "Forbidden" triggers if update_message returns 0,
        // but find_by_id then *does* find the donation. This implies the user_id in update_message
        // didn't match, or some other condition prevented the update for that user.

        mock_donation_repo.expect_update_message(move |d_id, u_id, msg| {
            assert_eq!(d_id, donation_id_exists);
            assert_eq!(u_id, user_id_performing_delete); // Service passes the DUMMY_USER_ID
            assert_eq!(msg, None);
            Ok(0) // Simulate 0 rows affected (e.g., user_id didn't match in a WHERE clause)
        });

        let existing_donation = sample_donation(donation_id_exists, 456, 1, 50); // Owned by a different user
        mock_donation_repo.expect_find_by_id({
            let ed = existing_donation.clone();
            move |d_id| {
                assert_eq!(d_id, donation_id_exists);
                Ok(Some(ed.clone())) // Donation found
            }
        });

        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");

        let response = client
            .delete(format!("/api/donations/{}/message", donation_id_exists))
            .dispatch();
        
        assert_eq!(response.status(), Status::Forbidden);
        let body = response.into_json::<serde_json::Value>().unwrap();
        assert_eq!(body["error"], "You cannot delete this donation message");
    }


    #[test]
    fn test_get_campaign_donations_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new(); // Not used in this service method

        let campaign_id_to_query = 1;
        let donations_list = vec![
            sample_donation(1, 10, campaign_id_to_query, 100),
            sample_donation(2, 20, campaign_id_to_query, 200)
        ];
        
        mock_donation_repo.expect_find_by_campaign({
            let dl = donations_list.clone();
            move |cid| {
                assert_eq!(cid, campaign_id_to_query);
                Ok(dl.clone())
            }
        });

        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");
        let response = client.get(format!("/api/campaigns/{}/donations", campaign_id_to_query)).dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Donation>>().unwrap();
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].id, donations_list[0].id);
    }
    
    #[test]
    fn test_get_campaign_donations_empty_result() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();

        let campaign_id_to_query = 2;
        mock_donation_repo.expect_find_by_campaign(move |cid| {
                assert_eq!(cid, campaign_id_to_query);
                Ok(vec![]) // Service returns an empty vector
            });

        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");
        let response = client.get(format!("/api/campaigns/{}/donations", campaign_id_to_query)).dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Donation>>().unwrap();
        assert!(body.is_empty());
    }

    #[test]
    fn test_get_my_donations_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();

        let user_id_querying = 123; // DUMMY_USER_ID from controller
        let donations_list = vec![
            sample_donation(1, user_id_querying, 1, 100),
            sample_donation(2, user_id_querying, 2, 50)
        ];

        mock_donation_repo.expect_find_by_user({
            let dl = donations_list.clone();
            move |uid| {
                assert_eq!(uid, user_id_querying);
                Ok(dl.clone())
            }
        });
        
        let donation_service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
        );
        let client = Client::tracked(create_test_rocket_with_service(donation_service)).expect("valid rocket instance");
        let response = client
            .get("/api/donations/me")
            // .header(Header::new("X-User-Id", user_id_querying.to_string())) // Header not used by DummyAuthUser
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Donation>>().unwrap();
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].user_id, user_id_querying);
    }
}