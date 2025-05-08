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
    use chrono::Utc;
    use mockall::mock; // Import the mock macro
    use mockall::predicate::*;
    use std::sync::Arc;

    // --- Local Mock Structures Implementing Actual Traits ---

    // Mock for DonationRepository
    mock! {
        pub TestDonationRepo { // Struct name can be anything
            // No need to list methods here if they match the trait
        }

        #[async_trait]
        impl DonationRepository for TestDonationRepo {
            async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
            async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
            async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
            async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
            async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
        }
    }

    // Mock for CampaignRepository
    mock! {
        pub TestCampaignRepo { // Struct name can be anything
            // No need to list methods here if they match the trait
        }

        #[async_trait]
        impl CampaignRepository for TestCampaignRepo {
            // Method used by DonationService and tested
            async fn get_campaign(&self, campaign_id: i32) -> Result<Option<Campaign>, AppError>;

            // Add declarations for all other methods from the CampaignRepository trait
            // Even if not directly called by DonationService in these tests,
            // the mock needs to acknowledge their existence.
            async fn create_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>;
            async fn update_campaign(&self, campaign: Campaign) -> Result<Campaign, AppError>;
            async fn update_campaign_status(&self, id: i32, status: CampaignStatus) -> Result<bool, AppError>;
            async fn get_campaigns_by_user(&self, user_id: i32) -> Result<Vec<Campaign>, AppError>;
            async fn get_campaigns_by_status(&self, status: CampaignStatus) -> Result<Vec<Campaign>, AppError>;
        }
    }

    // Helper to create a dummy campaign for tests
    fn dummy_campaign(id: i32, status: CampaignStatus) -> Campaign {
        Campaign {
            id,
            user_id: 1,
            name: "Test Campaign".to_string(),
            description: "A campaign for testing donations".to_string(),
            target_amount: 1000.0,
            collected_amount: 0.0,
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_url: None,
            evidence_uploaded_at: None,
        }
    }

    // --- Test Cases ---

    #[tokio::test]
    async fn test_make_donation_success() {
        let mut mock_donation_repo = MockTestDonationRepo::new(); // Use the generated mock name
        let mut mock_campaign_repo = MockTestCampaignRepo::new(); // Use the generated mock name
        let donor_id = 1;
        let campaign_id = 10;
        let amount = 50.0;

        let campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        let campaign_clone = campaign.clone();

        let expected_donation = Donation {
            id: 1, // Assumed ID from repo
            user_id: donor_id,
            campaign_id,
            amount,
            message: None,
            created_at: Utc::now(), // Actual value will come from mock
        };
        let expected_donation_clone = expected_donation.clone();

        mock_campaign_repo
            .expect_get_campaign() // Use the correct method name
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(Some(campaign_clone.clone())));

        mock_donation_repo
            .expect_create()
            .withf(move |uid, req: &NewDonationRequest| {
                *uid == donor_id && req.campaign_id == campaign_id && req.amount == amount && req.message.is_none()
            })
            .times(1)
            .returning(move |_, _| Ok(expected_donation_clone.clone()));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));

        let cmd = MakeDonationCommand {
            donor_id,
            campaign_id,
            amount,
            message: None,
        };
        let result = service.make_donation(cmd).await;

        assert!(result.is_ok());
        let donation = result.unwrap();
        assert_eq!(donation.id, expected_donation.id); // Compares the whole struct
        assert_eq!(donation.user_id, donor_id);
        assert_eq!(donation.amount, amount);
    }

    #[tokio::test]
    async fn test_make_donation_invalid_amount() {
        let mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));

        let cmd = MakeDonationCommand {
            donor_id: 1,
            campaign_id: 10,
            amount: 0.0,
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
        let campaign_id = 99;

        mock_campaign_repo
            .expect_get_campaign() // Use the correct method name
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(None));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
        let cmd = MakeDonationCommand {
            donor_id: 1,
            campaign_id,
            amount: 50.0,
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
        let campaign_id = 10;

        mock_campaign_repo
            .expect_get_campaign() // Use the correct method name
            .with(eq(campaign_id))
            .times(1)
            .returning(|_| Err(AppError::InvalidOperation("Simulated DB Error from CampaignRepo".to_string())));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));

        let cmd = MakeDonationCommand {
            donor_id: 1,
            campaign_id,
            amount: 50.0,
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
        let donor_id = 1;
        let campaign_id = 10;
        let amount = 50.0;

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
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));

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
        let donation_id = 5;
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(1)); // Simulate 1 row affected

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
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
            amount: 50.0,
            message: Some("Test".to_string()),
            created_at: Utc::now(),
        };
        mock_donation_repo
            .expect_find_by_id()
            .with(eq(donation_id))
            .times(1)
            .returning(move |_| Ok(Some(existing_donation.clone())));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
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
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
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
        let donation_id = 5;
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Err(AppError::InvalidOperation("Simulated DB Error on update_message".to_string())));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
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
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
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
        let campaign_id = 10;
        let expected_donations = vec![
            Donation {
                id: 1, user_id: 1, campaign_id, amount: 50.0, message: None, created_at: Utc::now(),
            },
            Donation {
                id: 2, user_id: 2, campaign_id, amount: 100.0, message: Some("Good luck!".to_string()), created_at: Utc::now(),
            },
        ];
        let expected_donations_clone = expected_donations.clone();

        mock_donation_repo
            .expect_find_by_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(expected_donations_clone.clone()));

        let service = DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
        let result = service.get_donations_by_campaign(campaign_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_donations);
    }


    #[tokio::test]
    async fn test_get_donations_by_campaign_empty() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let campaign_id = 10;

        mock_donation_repo
            .expect_find_by_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
        let result = service.get_donations_by_campaign(campaign_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_donations_by_campaign_repo_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let campaign_id = 10;

        mock_donation_repo
            .expect_find_by_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(|_| Err(AppError::InvalidOperation("Simulated DB Error on find_by_campaign".to_string())));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
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
        let user_id = 1;
        let expected_donations = vec![
            Donation { id: 1, user_id, campaign_id: 10, amount: 50.0, message: None, created_at: Utc::now() },
            Donation { id: 2, user_id, campaign_id: 11, amount: 75.0, message: Some("Hi".to_string()), created_at: Utc::now() },
        ];
        let expected_donations_clone = expected_donations.clone();

        mock_donation_repo
            .expect_find_by_user()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(expected_donations_clone.clone()));

        let service = DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
        let result = service.get_donations_by_user(user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_donations);
    }

    #[tokio::test]
    async fn test_get_donations_by_user_empty() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let user_id = 1;

        mock_donation_repo
            .expect_find_by_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
        let result = service.get_donations_by_user(user_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_donations_by_user_repo_error() {
        let mut mock_donation_repo = MockTestDonationRepo::new();
        let mock_campaign_repo = MockTestCampaignRepo::new();
        let user_id = 1;

        mock_donation_repo
            .expect_find_by_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(AppError::InvalidOperation("Simulated DB Error on find_by_user".to_string())));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
        let result = service.get_donations_by_user(user_id).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            AppError::InvalidOperation(msg) => assert_eq!(msg, "Simulated DB Error on find_by_user"),
            e => panic!("Expected InvalidOperation, got {:?}", e),
        }
    }
}