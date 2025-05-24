use crate::errors::AppError;
use crate::model::donation::donation::{Donation, NewDonationRequest};
use crate::repository::campaign::campaign_repository::CampaignRepository;
use crate::repository::donation::donation_repository::DonationRepository;
use crate::service::commands::donation_commands::{
    DeleteDonationMessageCommand, MakeDonationCommand,
};
use crate::model::campaign::campaign::CampaignStatus; // Added for CampaignStatus::Active
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

        // Fetch the campaign
        let mut campaign = self
            .campaign_repo
            .get_campaign(cmd.campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        // Check if the campaign is active
        if campaign.status != CampaignStatus::Active {
            return Err(AppError::InvalidOperation(
                "Donations can only be made to active campaigns".to_string(),
            ));
        }

        // Prepare the new donation request
        let req = NewDonationRequest {
            campaign_id: cmd.campaign_id,
            amount: cmd.amount,
            message: cmd.message,
        };

        // Create the donation
        // IMPORTANT: In a real-world scenario with database-backed repositories,
        // the creation of the donation and the update of the campaign's collected amount
        // should be performed within a single database transaction to ensure atomicity.
        // This would typically involve modifying repository methods to accept a transaction
        // handle or managing the transaction at the service layer if the pool is directly accessible.
        let donation = self.donation_repo.create(cmd.donor_id, &req).await?;

        // Update the campaign's collected amount
        campaign.collected_amount += donation.amount;

        // Persist the updated campaign information
        match self.campaign_repo.update_campaign(campaign.clone()).await { // campaign.clone() because update_campaign takes ownership
            Ok(_) => {
                // Optionally, check if campaign needs to be marked as Completed
                if campaign.collected_amount >= campaign.target_amount && campaign.status == CampaignStatus::Active {
                    // This part would ideally be handled by the CampaignService or a dedicated
                    // CampaignState transition mechanism, potentially triggered by an event or observer,
                    // rather than directly in DonationService.
                    // For now, we'll just note it. A more robust solution would involve
                    // self.campaign_repo.update_campaign_status(campaign.id, CampaignStatus::Completed).await?;
                    // or calling a CampaignService method.
                    println!(
                        "Campaign {} has reached its target of {}. Collected: {}",
                        campaign.id, campaign.target_amount, campaign.collected_amount
                    );
                }
                Ok(donation)
            }
            Err(e) => {
                // This is a critical state: donation was created, but campaign update failed.
                // Log this error prominently. A compensation mechanism (e.g., trying to delete
                // the donation or flagging it for manual review) might be needed in a robust system.
                eprintln!(
                    "CRITICAL ERROR: Donation ID {} was created for campaign ID {}, but failed to update campaign's collected amount. Error: {:?}",
                    donation.id, cmd.campaign_id, e
                );
                // Return an error indicating a partial failure.
                Err(AppError::InternalServerError(format!(
                    "Donation created, but failed to update campaign (ID: {}): {}",
                    cmd.campaign_id, e
                )))
            }
        }
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
                // This means the donation exists but either the user_id didn't match
                // or the message was already None (though update should still report 1 row affected if it matches).
                // More likely, the user_id didn't match.
                return Err(AppError::Forbidden(
                    "You cannot delete this donation message or donation not found for user".to_string(),
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