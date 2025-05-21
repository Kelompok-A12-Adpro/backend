use crate::errors::AppError;
use crate::model::donation::donation::Donation;
use crate::model::donation::donation::NewDonationRequest;
use crate::repository::campaign::campaign_repository::CampaignRepository;
use crate::repository::donation::donation_repository::DonationRepository;
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
            .get_campaign(cmd.campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        let req = NewDonationRequest {
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