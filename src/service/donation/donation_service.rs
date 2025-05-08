use crate::errors::AppError;
use crate::model::donation::Donation;
use crate::repository::campaign_repo::CampaignRepository;
use crate::repository::donation_repo::DonationRepository;
use crate::service::commands::donation_commands::{
    DeleteDonationMessageCommand, MakeDonationCommand,
};
use std::sync::Arc;

pub struct DonationService {
    donation_repo: Arc<dyn DonationRepository>,
    campaign_repo: Arc<dyn CampaignRepository>,
}

impl DonationService {
    pub fn new(
        donation_repo: Arc<dyn DonationRepository>,
        campaign_repo: Arc<dyn CampaignRepository>,
    ) -> Self {
        DonationService {
            donation_repo,
            campaign_repo,
        }
    }

    pub async fn make_donation(&self, cmd: MakeDonationCommand) -> Result<Donation, AppError> {
        if cmd.amount <= 0.0 {
            return Err(AppError::ValidationError(
                "Donation amount must be positive".to_string(),
            ));
        }

        let campaign = self
            .campaign_repo
            .find_by_id(cmd.campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        let req = crate::model::donation::NewDonationRequest {
            campaign_id: cmd.campaign_id,
            amount: cmd.amount,
            message: cmd.message,
        };

        let donation = self.donation_repo.create(cmd.donor_id, &req).await?;

        Ok(donation)
    }

    pub async fn delete_donation_message(
        &self,
        cmd: DeleteDonationMessageCommand,
    ) -> Result<(), AppError> {
        let rows_affected = self
            .donation_repo
            .update_message(cmd.donation_id, cmd.user_id, None)
            .await?;

        if rows_affected == 0 {
            let donation_exists = self
                .donation_repo
                .find_by_id(cmd.donation_id)
                .await?
                .is_some();
            if !donation_exists {
                return Err(AppError::NotFound("Donation not found".to_string()));
            } else {
                return Err(AppError::Forbidden(
                    "You cannot delete this donation message".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub async fn get_donations_by_campaign(
        &self,
        campaign_id: i32,
    ) -> Result<Vec<Donation>, AppError> {
        self.donation_repo.find_by_campaign(campaign_id).await
    }

    pub async fn get_donations_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError> {
        self.donation_repo.find_by_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::model::{campaign::Campaign, donation::Donation};
    use crate::repository::{
        campaign_repo::{CampaignRepository, MockCampaignRepository},
        donation_repo::{DonationRepository, MockDonationRepository},
    };
    use chrono::Utc;
    use mockall::predicate::*;
    use std::sync::Arc;

    fn create_mock_service() -> (
        DonationService,
        MockDonationRepository,
        MockCampaignRepository,
    ) {
        let mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));

        let mut mock_donation_repo = MockDonationRepository::new();
        let mut mock_campaign_repo = MockCampaignRepository::new();

        let donation_repo_arc = Arc::new(MockDonationRepository::new());
        let campaign_repo_arc = Arc::new(MockCampaignRepository::new());
        let service = DonationService::new(donation_repo_arc.clone(), campaign_repo_arc.clone());

        (
            service,
            Arc::try_unwrap(donation_repo_arc).ok().unwrap(),
            Arc::try_unwrap(campaign_repo_arc).ok().unwrap(),
        )
    }

    #[tokio::test]
    async fn test_make_donation_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mut mock_campaign_repo = MockCampaignRepository::new();
        let donor_id = 1;
        let campaign_id = 10;
        let amount = 50.0;

        let expected_campaign = Campaign {
            /* ... fill campaign details ... */ id: campaign_id, /* status: Active */
        };
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
            .returning(move |_| Ok(Some(expected_campaign.clone())));

        mock_donation_repo
            .expect_create()
            .withf(move |uid, req| {
                *uid == donor_id && req.campaign_id == campaign_id && req.amount == amount
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
            AppError::ValidationError(msg) => assert!(msg.contains("must be positive")),
            _ => panic!("Expected ValidationError"),
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
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_donation_message_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
        let donation_id = 5;
        let user_id = 1;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(1));

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
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
        let donation_id = 5;
        let owner_user_id = 1;
        let attacker_user_id = 2;

        mock_donation_repo
            .expect_update_message()
            .with(eq(donation_id), eq(attacker_user_id), eq(None::<String>))
            .times(1)
            .returning(|_, _, _| Ok(0));

        let existing_donation = Donation {
            id: donation_id,
            user_id: owner_user_id,
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
            AppError::Forbidden(msg) => {
                assert!(msg.contains("cannot delete this donation message"))
            }
            _ => panic!("Expected Forbidden error"),
        }
    }

    #[tokio::test]
    async fn test_delete_donation_message_not_found() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
        let donation_id = 99;
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
            .returning(move |_| Ok(None));

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
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_donations_by_campaign_success() {
        let mut mock_donation_repo = MockDonationRepository::new();
        let mock_campaign_repo = MockCampaignRepository::new();
        let campaign_id = 10;
        let expected_donations = vec![
            Donation {
                id: 1,
                user_id: 1,
                campaign_id,
                amount: 50.0,
                message: None,
                created_at: Utc::now(),
            },
            Donation {
                id: 2,
                user_id: 2,
                campaign_id,
                amount: 100.0,
                message: Some("Good luck!".to_string()),
                created_at: Utc::now(),
            },
        ];
        let expected_donations_clone = expected_donations.clone();

        mock_donation_repo
            .expect_find_by_campaign()
            .with(eq(campaign_id))
            .times(1)
            .returning(move |_| Ok(expected_donations_clone.clone()));

        let service =
            DonationService::new(Arc::new(mock_donation_repo), Arc::new(mock_campaign_repo));
        let result = service.get_donations_by_campaign(campaign_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_donations);
    }
    // Add test for get_donations_by_user...
}

// IMPORTANT: You need to derive the Mock structs for your repository traits.
// Add this near your trait definitions or in a separate test utility module.
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CampaignRepository: Send + Sync {
    async fn find_by_id(&self, campaign_id: i32) -> Result<Option<Campaign>, AppError>;
}