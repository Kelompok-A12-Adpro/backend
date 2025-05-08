#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::model::{
        campaign::campaign::{Campaign, CampaignStatus},
        donation::{Donation, NewDonationRequest},
    };
    use crate::repository::{
        campaign_repo::{MockCampaignRepository, CampaignRepository as _TraitCampaignRepo},
        donation_repo::{MockDonationRepository, DonationRepository as _TraitDonationRepo},
    };
    use crate::service::commands::donation_commands::{
        MakeDonationCommand, DeleteDonationMessageCommand,
    };
    use chrono::Utc;
    use mockall::predicate::*;
    use std::sync::Arc;

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

    #[tokio::test]
    async fn test_make_donation_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mut mock_campaign_repo = MockCampaignRepository::new();
        let donor_id = 1;
        let campaign_id = 10;
        let amount = 50.0;

        let campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        let campaign_clone = campaign.clone();

        let expected_donation = Donation {
            id: 1,
            user_id: donor_id,
            campaign_id,
            amount,
            message: None,
            created_at: Utc::now(),
        };
        let expected_donation_clone = expected_donation.clone();

        mock_campaign_repo
            .expect_find_by_id()
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
        assert_eq!(donation.id, expected_donation.id);
        assert_eq!(donation.user_id, donor_id);
        assert_eq!(donation.amount, amount);
    }

    #[tokio::test]
    async fn test_make_donation_invalid_amount() {
        let mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
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
        let mut mock_donation_repo = MockDonationRepository::new();
        let mut mock_campaign_repo = MockCampaignRepository::new();
        let campaign_id = 99;

        mock_campaign_repo
            .expect_find_by_id()
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
        let mut mock_donation_repo = MockDonationRepository::new();
        let mut mock_campaign_repo = MockCampaignRepository::new();
        let campaign_id = 10;

        mock_campaign_repo
            .expect_find_by_id()
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
        let mut mock_donation_repo = MockDonationRepository::new();
        let mut mock_campaign_repo = MockCampaignRepository::new();
        let donor_id = 1;
        let campaign_id = 10;
        let amount = 50.0;

        let campaign = dummy_campaign(campaign_id, CampaignStatus::Active);
        let campaign_clone = campaign.clone();

        mock_campaign_repo
            .expect_find_by_id()
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
    async fn test_delete_donation_message_repo_update_error() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
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
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
        let donation_id = 5;
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(0));

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
    async fn test_get_donations_by_campaign_empty() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
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
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
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
    async fn test_get_donations_by_user_empty() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
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
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
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