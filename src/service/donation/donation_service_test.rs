#[cfg(test)]
mod tests {
    use crate::service::donation::donation_service::DonationService;
    use crate::errors::AppError;
    use crate::model::{
        campaign::campaign::{Campaign, CampaignStatus},
        donation::donation::{Donation, NewDonationRequest},
    };

    use crate::repository::{
        campaign::campaign_repository::CampaignRepository,
        donation::donation_repository::DonationRepository,
    };
    use crate::service::commands::donation_commands::{
        MakeDonationCommand, DeleteDonationMessageCommand,
    };
    use async_trait::async_trait;
    use chrono::{Utc, Duration};
    use mockall::mock; // Import the mock macro
    use mockall::predicate::*;
    use std::sync::Arc;

    
    use crate::model::wallet::wallet::Wallet; // Make sure Wallet is imported

    // CRITICAL: This 'use' statement must correctly point to your WalletRepository trait definition
    use crate::repository::wallet::wallet_repository::WalletRepository;

    // --- Local Mock Structures Implementing Actual Traits ---

    // Mock for DonationRepository
    mock! {
        pub TestDonationRepo { // Struct name can be anything
        }

        #[async_trait]
        impl DonationRepository for TestDonationRepo {
            async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
            async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
            async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
            async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
            async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
            // Added missing methods from the trait to the mock definition
            async fn get_total_donated_for_campaign(&self, campaign_id: i32) -> Result<i64, AppError>;
            async fn get_user_total_for_campaign(&self, user_id: i32, campaign_id: i32) -> Result<i64, AppError>;
        }
    }

    // Mock for CampaignRepository
    mock! {
        pub TestCampaignRepo {
        }

        #[async_trait]
        impl CampaignRepository for TestCampaignRepo {
            async fn create_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>;
            async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError>;
            async fn update_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>; // Still used for general updates if any
            async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError>;
            async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError>;
            async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError>;
            async fn get_all_campaigns(&self) -> Result<Vec<Campaign>, AppError>;
            async fn delete_campaign(&self, id: i32) -> Result<bool, AppError>;
        }
    }

    mock! {
        pub TestWalletRepo { // Struct name can be anything, e.g., MockWalletRepository
            // No need to list methods here if they match the trait and you're using this syntax
        }

        #[async_trait]
        impl WalletRepository for TestWalletRepo {
            async fn find_by_user_id(&self, user_id: i32) -> Result<Option<Wallet>, AppError>;
            async fn update_balance(&self, user_id: i32, new_balance: f64) -> Result<(), AppError>;
            async fn create_wallet_if_not_exists(&self, user_id: i32) -> Result<Wallet, AppError>;
        }
    }

    // Helper to create a dummy wallet for tests if needed in expectations
    fn dummy_wallet(id: i32, user_id: i32, balance: f64) -> Wallet {
        Wallet {
            id,
            user_id,
            balance,
            updated_at: Utc::now().naive_utc(),
        }
    }

    // Helper to create a dummy campaign for tests
    fn dummy_campaign(id: i32, status: CampaignStatus) -> Campaign {
        let now = Utc::now();
        Campaign {
            id,
            user_id: 1,
            name: "Test Campaign".to_string(),
            description: "A campaign for testing donations".to_string(),
            target_amount: 1000,
            collected_amount: 0,
            start_date: now, // Added
            end_date: now + Duration::days(30), // Added - example: 30 days from now
            image_url: Some("http://example.com/default_campaign_image.png".to_string()), // Added - example URL
            evidence_url: None,
            evidence_uploaded_at: None,
            status,
            created_at: now,
            updated_at: now,
        }
    }

    // --- Test Cases ---

        #[tokio::test]
    async fn test_make_donation_success_target_not_met() { // Renamed for clarity
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donor_id = 1;
        let campaign_id = 10;
        let donation_amount = 50;
        let initial_collected_amount = 100;
        let campaign_target_amount = 1000; // Ensure target is not met

        // --- Initial Campaign State ---
        let mut initial_campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        initial_campaign.collected_amount = initial_collected_amount;
        initial_campaign.target_amount = campaign_target_amount;
        let initial_campaign_clone = initial_campaign.clone();

        // --- Expected Donation from repo.create ---
        let expected_donation = Donation {
            id: 1,
            user_id: donor_id,
            campaign_id,
            amount: donation_amount,
            message: None,
            created_at: Utc::now(), // Mock will return this structure
        };
        let expected_donation_clone = expected_donation.clone();

        // --- Updated Campaign State (after donation repo.create implicitly updates it) ---
        let mut updated_campaign_after_donation = initial_campaign.clone();
        updated_campaign_after_donation.collected_amount = initial_collected_amount + donation_amount;
        let updated_campaign_clone = updated_campaign_after_donation.clone();


        // 1. Expect get_campaign (for initial check)
        mock_campaign_repo
            .expect_get_campaign()
            .with(eq(campaign_id))
            .times(1) // Called once for initial check
            .returning(move |_| Ok(Some(initial_campaign_clone.clone())));

        // 2. Expect donation_repo.create to be called
        // This now handles wallet debit, donation creation, and campaign collected_amount update
        mock_donation_repo
            .expect_create()
            .withf(move |uid, req: &NewDonationRequest| {
                *uid == donor_id && req.campaign_id == campaign_id && req.amount == donation_amount && req.message.is_none()
            })
            .times(1)
            .returning(move |_, _| Ok(expected_donation_clone.clone()));

        // 3. Expect get_campaign AGAIN (to fetch updated campaign state after donation)
        mock_campaign_repo
            .expect_get_campaign()
            .with(eq(campaign_id))
            .times(1) // Called again to get updated state
            .returning(move |_| Ok(Some(updated_campaign_clone.clone())));

        // 4. update_campaign_status should NOT be called as target is not met
        mock_campaign_repo.expect_update_campaign_status().never();
        // 5. update_campaign should NOT be called by the service for collected_amount
        mock_campaign_repo.expect_update_campaign().never();


        let service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
            Arc::new(mock_wallet_repo)
        );

        let cmd = MakeDonationCommand {
            donor_id,
            campaign_id,
            amount: donation_amount,
            message: None,
        };
        let result = service.make_donation(cmd).await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        let donation_received = result.unwrap();
        assert_eq!(donation_received.id, expected_donation.id);
        assert_eq!(donation_received.user_id, donor_id);
        assert_eq!(donation_received.amount, donation_amount);
    }

    #[tokio::test]
    async fn test_make_donation_invalid_amount() {
        let mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));

        let cmd = MakeDonationCommand {
            donor_id: 1,
            campaign_id: 10,
            amount: 0,
            message: None,
        };
        let result = service.make_donation(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::ValidationError(msg) => assert!(msg.contains("Donation amount must be positive")),
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_make_donation_campaign_not_found() {
        let mock_donation_repo = MockTestDonationRepo::new();
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let campaign_id = 99;

        mock_campaign_repo
            .expect_get_campaign() // Use the correct method name
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(None));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo),
                Arc::new(mock_wallet_repo));
        let cmd = MakeDonationCommand {
            donor_id: 1,
            campaign_id,
            amount: 50,
            message: None,
        };
        let result = service.make_donation(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::NotFound(msg) => assert!(msg.contains("Campaign not found")),
            e => panic!("Expected NotFound error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_make_donation_campaign_repo_error() {
        let mock_donation_repo = MockTestDonationRepo::new();
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let campaign_id = 10;

        mock_campaign_repo
            .expect_get_campaign() // Use the correct method name
            .with(eq(campaign_id))
            .times(1)
            .returning(|_| Err(AppError::InvalidOperation("Simulated DB Error from CampaignRepo".to_string())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo),
                Arc::new(mock_wallet_repo));

        let cmd = MakeDonationCommand {
            donor_id: 1,
            campaign_id,
            amount: 50,
            message: None,
        };
        let result = service.make_donation(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => assert_eq!(msg, "Simulated DB Error from CampaignRepo"),
            e => panic!("Expected InvalidOperation error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_make_donation_donation_repo_create_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donor_id = 1;
        let campaign_id = 10;
        let amount = 50;

        let campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        let campaign_clone = campaign.clone();

        mock_campaign_repo
            .expect_get_campaign() // Use the correct method name
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(Some(campaign_clone.clone())));

        mock_donation_repo
            .expect_create()
            .withf(move |uid, req: &NewDonationRequest| {
                *uid == donor_id && req.campaign_id == campaign_id && req.amount == amount
            })
            .times(1)
            .returning(|_, _| Err(AppError::InvalidOperation("Simulated DB Error from DonationRepo create".to_string())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));

        let cmd = MakeDonationCommand {
            donor_id,
            campaign_id,
            amount,
            message: None,
        };
        let result = service.make_donation(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => assert_eq!(msg, "Simulated DB Error from DonationRepo create"),
            e => panic!("Expected InvalidOperation error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_donation_message_success() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new(); // Instantiated but not directly used in this test's logic path
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donation_id = 5;
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(1)); // Simulate 1 row affected

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let cmd = DeleteDonationMessageCommand {
            donation_id,
            user_id,
        };
        let result = service.delete_donation_message(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_donation_message_forbidden() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donation_id = 5;
        let owner_user_id = 1; // The actual owner
        let attacker_user_id = 2; // User trying to delete who is not the owner

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(attacker_user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(0)); // Simulate 0 rows affected (e.g. DB query had a WHERE user_id = attacker_user_id)

        // Mock find_by_id to confirm the donation exists, so the error becomes Forbidden, not NotFound
        let existing_donation = Donation {
            id: donation_id,
            user_id: owner_user_id, // Donation belongs to owner_user_id
            campaign_id: 10,
            amount: 50,
            message: Some("Test".to_string()),
            created_at: Utc::now(),
        };
        mock_donation_repo
            .expect_find_by_id()
            .with(eq(donation_id))
            .times(1)
            .returning(move |_| Ok(Some(existing_donation.clone())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let cmd = DeleteDonationMessageCommand {
            donation_id,
            user_id: attacker_user_id,
        };
        let result = service.delete_donation_message(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::Forbidden(msg) => assert!(msg.contains("You cannot delete this donation message")),
            e => panic!("Expected Forbidden error, got {:?}", e),
        }
    }


    #[tokio::test]
    async fn test_delete_donation_message_not_found_after_update_fails() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donation_id = 99; // Non-existent donation
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(0)); // 0 rows affected as donation_id doesn't exist or user_id doesn't match

        // Mock find_by_id to return None, confirming donation doesn't exist
        mock_donation_repo
            .expect_find_by_id()
            .with(eq(donation_id))
            .times(1)
            .returning(|_| Ok(None));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let cmd = DeleteDonationMessageCommand {
            donation_id,
            user_id,
        };
        let result = service.delete_donation_message(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::NotFound(msg) => assert!(msg.contains("Donation not found")),
            e => panic!("Expected NotFound error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_donation_message_repo_update_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donation_id = 5;
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Err(AppError::InvalidOperation("Simulated DB Error on update_message".to_string())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let cmd = DeleteDonationMessageCommand {
            donation_id,
            user_id,
        };
        let result = service.delete_donation_message(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => assert_eq!(msg, "Simulated DB Error on update_message"),
            e => panic!("Expected InvalidOperation, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_donation_message_repo_find_by_id_error_after_update_zero() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donation_id = 5;
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(0)); // Simulate no rows affected

        mock_donation_repo
            .expect_find_by_id()
            .with(eq(donation_id))
            .times(1)
            .returning(|_| Err(AppError::InvalidOperation("Simulated DB Error on find_by_id".to_string())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let cmd = DeleteDonationMessageCommand {
            donation_id,
            user_id,
        };
        let result = service.delete_donation_message(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => assert_eq!(msg, "Simulated DB Error on find_by_id"),
            e => panic!("Expected InvalidOperation, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_donations_by_campaign_success() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new(); // Not directly used in this path
        let mock_wallet_repo = MockTestWalletRepo::new();
        let campaign_id = 10;
        let expected_donations = vec![
            Donation {
                id: 1, user_id: 1, campaign_id, amount: 50, message: None, created_at: Utc::now(),
            },
            Donation {
                id: 2, user_id: 2, campaign_id, amount: 100, message: Some("Good luck!".to_string()), created_at: Utc::now(),
            },
        ];
        let expected_donations_clone = expected_donations.clone();

        mock_donation_repo
            .expect_find_by_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(expected_donations_clone.clone()));

        let service = DonationService::new(
            Arc::new(mock_donation_repo), 
            Arc::new(mock_campaign_repo), 
            Arc::new(mock_wallet_repo));
        let result = service.get_donations_by_campaign(campaign_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_donations);
    }


    #[tokio::test]
    async fn test_get_donations_by_campaign_empty() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let campaign_id = 10;

        mock_donation_repo
            .expect_find_by_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let result = service.get_donations_by_campaign(campaign_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_donations_by_campaign_repo_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let campaign_id = 10;

        mock_donation_repo
            .expect_find_by_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(|_| Err(AppError::InvalidOperation("Simulated DB Error on find_by_campaign".to_string())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo),
                Arc::new(mock_wallet_repo));
        let result = service.get_donations_by_campaign(campaign_id).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => assert_eq!(msg, "Simulated DB Error on find_by_campaign"),
            e => panic!("Expected InvalidOperation, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_donations_by_user_success() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new(); // Not directly used
        let mock_wallet_repo = MockTestWalletRepo::new();
        let user_id = 1;
        let expected_donations = vec![
            Donation { id: 1, user_id, campaign_id: 10, amount: 50, message: None, created_at: Utc::now() },
            Donation { id: 2, user_id, campaign_id: 11, amount: 75, message: Some("Hi".to_string()), created_at: Utc::now() },
        ];
        let expected_donations_clone = expected_donations.clone();

        mock_donation_repo
            .expect_find_by_user()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(expected_donations_clone.clone()));

        let service = DonationService::new(
            Arc::new(mock_donation_repo), 
            Arc::new(mock_campaign_repo), 
            Arc::new(mock_wallet_repo));
        let result = service.get_donations_by_user(user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_donations);
    }

    #[tokio::test]
    async fn test_get_donations_by_user_empty() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let user_id = 1;

        mock_donation_repo
            .expect_find_by_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let result = service.get_donations_by_user(user_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_donations_by_user_repo_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let user_id = 1;

        mock_donation_repo
            .expect_find_by_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(AppError::InvalidOperation("Simulated DB Error on find_by_user".to_string())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));
        let result = service.get_donations_by_user(user_id).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => assert_eq!(msg, "Simulated DB Error on find_by_user"),
            e => panic!("Expected InvalidOperation, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_make_donation_campaign_not_active() {
        let mock_donation_repo = MockTestDonationRepo::new(); // Not expected to be called
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let campaign_id = 10;

        // Campaign is PendingVerification, not Active
        let campaign = dummy_campaign(campaign_id, CampaignStatus::PendingVerification);

        mock_campaign_repo
            .expect_get_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(Some(campaign.clone())));

        let service =
            DonationService::new(
                Arc::new(mock_donation_repo), 
                Arc::new(mock_campaign_repo), 
                Arc::new(mock_wallet_repo));

        let cmd = MakeDonationCommand {
            donor_id: 1,
            campaign_id,
            amount: 50,
            message: None,
        };
        let result = service.make_donation(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => {
                assert!(msg.contains("Donations can only be made to active campaigns"))
            }
            e => panic!("Expected InvalidOperation for non-active campaign, got {:?}", e),
        }
    }


    #[tokio::test]
    async fn test_make_donation_success_and_target_met() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donor_id = 1;
        let campaign_id = 10;
        let donation_amount = 50;
        let initial_collected_amount = 950; // Will meet target of 1000
        let campaign_target_amount = 1000;

        let mut initial_campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        initial_campaign.collected_amount = initial_collected_amount;
        initial_campaign.target_amount = campaign_target_amount;
        let initial_campaign_clone = initial_campaign.clone();

        let expected_donation = Donation {
            id: 1, user_id: donor_id, campaign_id, amount: donation_amount, message: None, created_at: Utc::now(),
        };
        let expected_donation_clone = expected_donation.clone();

        let mut updated_campaign_after_donation = initial_campaign.clone();
        updated_campaign_after_donation.collected_amount = initial_collected_amount + donation_amount; // Now 1000
        // Status is still Active when re-fetched, before update_campaign_status is called
        let updated_campaign_clone = updated_campaign_after_donation.clone();

        // --- Mock Expectations ---
        // 1. Initial get_campaign
        mock_campaign_repo.expect_get_campaign()
            .with(eq(campaign_id)).times(1)
            .returning(move |_| Ok(Some(initial_campaign_clone.clone())));

        // 2. donation_repo.create
        mock_donation_repo.expect_create()
            .withf(move |uid, req: &NewDonationRequest| *uid == donor_id && req.campaign_id == campaign_id && req.amount == donation_amount)
            .times(1)
            .returning(move |_, _| Ok(expected_donation_clone.clone()));

        // 3. Second get_campaign (re-fetch)
        mock_campaign_repo.expect_get_campaign()
            .with(eq(campaign_id)).times(1)
            .returning(move |_| Ok(Some(updated_campaign_clone.clone())));

        // 4. update_campaign_status IS called because target is met
        mock_campaign_repo.expect_update_campaign_status()
            .with(eq(campaign_id), eq(CampaignStatus::Completed))
            .times(1)
            .returning(|_, _| Ok(true)); // Simulate successful status update

        let service = DonationService::new(
            Arc::new(mock_donation_repo), 
            Arc::new(mock_campaign_repo), 
            Arc::new(mock_wallet_repo));
        let cmd = MakeDonationCommand { donor_id, campaign_id, amount: donation_amount, message: None };
        let result = service.make_donation(cmd).await;

        assert!(result.is_ok());
        // Further assertions if needed on the returned donation
    }

        #[tokio::test]
    async fn test_make_donation_target_met_but_status_update_fails() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donor_id = 1;
        let campaign_id = 10;
        let donation_amount = 50;
        let initial_collected_amount = 950;
        let campaign_target_amount = 1000;

        let mut initial_campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        initial_campaign.collected_amount = initial_collected_amount;
        initial_campaign.target_amount = campaign_target_amount;
        let initial_campaign_clone = initial_campaign.clone();

        let expected_donation = Donation {
            id: 1, user_id: donor_id, campaign_id, amount: donation_amount, message: None, created_at: Utc::now(),
        };
        let expected_donation_clone = expected_donation.clone();

        let mut updated_campaign_after_donation = initial_campaign.clone();
        updated_campaign_after_donation.collected_amount = initial_collected_amount + donation_amount;
        let updated_campaign_clone = updated_campaign_after_donation.clone();

        // --- Mock Expectations ---
        mock_campaign_repo.expect_get_campaign()
            .with(eq(campaign_id)).times(1)
            .returning(move |_| Ok(Some(initial_campaign_clone.clone())));
        mock_donation_repo.expect_create()
            .times(1).returning(move |_, _| Ok(expected_donation_clone.clone()));
        mock_campaign_repo.expect_get_campaign()
            .with(eq(campaign_id)).times(1)
            .returning(move |_| Ok(Some(updated_campaign_clone.clone())));

        // update_campaign_status IS called but returns an error
        mock_campaign_repo.expect_update_campaign_status()
            .with(eq(campaign_id), eq(CampaignStatus::Completed))
            .times(1)
            .returning(|_, _| Err(AppError::DatabaseError("Failed to update status".to_string())));

        let service = DonationService::new(
            Arc::new(mock_donation_repo), 
            Arc::new(mock_campaign_repo), 
            Arc::new(mock_wallet_repo));
        let cmd = MakeDonationCommand { donor_id, campaign_id, amount: donation_amount, message: None };
        let result = service.make_donation(cmd).await;

        // The donation itself should still be successful
        assert!(result.is_ok(), "Donation should succeed even if status update fails. Got: {:?}", result.err());
        let donation_received = result.unwrap();
        assert_eq!(donation_received.id, expected_donation.id);
        // The service logs an eprintln! for this failure; testing stdout/stderr is more complex,
        // but we verify the primary operation succeeded.
    }

        #[tokio::test]
    async fn test_make_donation_refetch_campaign_fails() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mut mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let donor_id = 1;
        let campaign_id = 10;
        let donation_amount = 50;

        let initial_campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        let initial_campaign_clone = initial_campaign.clone();

        let expected_donation = Donation {
            id: 1, user_id: donor_id, campaign_id, amount: donation_amount, message: None, created_at: Utc::now(),
        };
        let expected_donation_clone = expected_donation.clone();

        // --- Mock Expectations ---
        // 1. Initial get_campaign
        mock_campaign_repo.expect_get_campaign()
            .with(eq(campaign_id)).times(1)
            .returning(move |_| Ok(Some(initial_campaign_clone.clone())));

        // 2. donation_repo.create succeeds
        mock_donation_repo.expect_create()
            .times(1)
            .returning(move |_, _| Ok(expected_donation_clone.clone()));

        // 3. Second get_campaign (re-fetch) FAILS
        mock_campaign_repo.expect_get_campaign()
            .with(eq(campaign_id)).times(1)
            .returning(move |_| Err(AppError::DatabaseError("DB error on re-fetch".to_string())));

        let service = DonationService::new(
            Arc::new(mock_donation_repo), 
            Arc::new(mock_campaign_repo), 
            Arc::new(mock_wallet_repo));
        let cmd = MakeDonationCommand { donor_id, campaign_id, amount: donation_amount, message: None };
        let result = service.make_donation(cmd).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::DatabaseError(msg) => assert_eq!(msg, "DB error on re-fetch"),
            e => panic!("Expected DatabaseError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_total_donated_for_campaign_from_repo_success() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new(); // Needed for service construction
        let mock_wallet_repo = MockTestWalletRepo::new();   // Needed for service construction
        let campaign_id = 123;
        let expected_total = 5000i64;

        mock_donation_repo
            .expect_get_total_donated_for_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(expected_total));

        let service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
            Arc::new(mock_wallet_repo),
        );

        let result = service
            .get_total_donated_for_campaign_from_repo(campaign_id)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_total);
    }

    #[tokio::test]
    async fn test_get_total_donated_for_campaign_from_repo_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let campaign_id = 123;

        mock_donation_repo
            .expect_get_total_donated_for_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(|_| Err(AppError::DatabaseError("Repo failed".to_string())));

        let service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
            Arc::new(mock_wallet_repo),
        );

        let result = service
            .get_total_donated_for_campaign_from_repo(campaign_id)
            .await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::DatabaseError(msg) => assert_eq!(msg, "Repo failed"),
            _ => panic!("Unexpected error type"),
        }
    }

    #[tokio::test]
    async fn test_get_my_total_for_campaign_from_repo_success() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let user_id = 1;
        let campaign_id = 123;
        let expected_total = 250i64;

        mock_donation_repo
            .expect_get_user_total_for_campaign()
            .with(eq(user_id), eq(campaign_id))
            .times(1)
            .returning(move |_, _| Ok(expected_total));

        let service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
            Arc::new(mock_wallet_repo),
        );

        let result = service
            .get_my_total_for_campaign_from_repo(user_id, campaign_id)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_total);
    }

    #[tokio::test]
    async fn test_get_my_total_for_campaign_from_repo_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let mock_wallet_repo = MockTestWalletRepo::new();
        let user_id = 1;
        let campaign_id = 123;

        mock_donation_repo
            .expect_get_user_total_for_campaign()
            .with(eq(user_id), eq(campaign_id))
            .times(1)
            .returning(|_, _| Err(AppError::NotFound("User total not found".to_string())));

        let service = DonationService::new(
            Arc::new(mock_donation_repo),
            Arc::new(mock_campaign_repo),
            Arc::new(mock_wallet_repo),
        );

        let result = service
            .get_my_total_for_campaign_from_repo(user_id, campaign_id)
            .await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::NotFound(msg) => assert_eq!(msg, "User total not found"),
            _ => panic!("Unexpected error type"),
        }
    }
}