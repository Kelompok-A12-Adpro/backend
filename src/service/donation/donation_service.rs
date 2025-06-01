use rocket::State;

use crate::errors::AppError;
use crate::model::admin::notification::CreateNotificationRequest;
use crate::model::donation::donation::{Donation, NewDonationRequest};
use crate::repository::campaign::campaign_repository::CampaignRepository;
use crate::repository::donation::donation_repository::DonationRepository;
use crate::service::admin::notification::notification_service::NotificationService;
use crate::service::commands::donation_commands::{
    DeleteDonationMessageCommand, MakeDonationCommand,
};
use crate::repository::wallet::wallet_repository::WalletRepository;
use crate::model::campaign::campaign::CampaignStatus; // Added for CampaignStatus::Active
use std::sync::Arc;

pub struct DonationService {
    donation_repo: Arc<dyn DonationRepository>,
    campaign_repo: Arc<dyn CampaignRepository>,
    wallet_repo: Arc<dyn WalletRepository>,
}

impl DonationService {
    pub fn new(
        donation_repo: Arc<dyn DonationRepository>,
        campaign_repo: Arc<dyn CampaignRepository>,
        wallet_repo: Arc<dyn WalletRepository>,
    ) -> Self {
        DonationService {
            donation_repo,
            campaign_repo,
            wallet_repo,
        }
    }

    pub async fn make_donation(&self, cmd: MakeDonationCommand, notification_service: &State<Arc<NotificationService>>) -> Result<Donation, AppError> {
        if cmd.amount <= 0 {
            return Err(AppError::ValidationError(
                "Donation amount must be positive".to_string(),
            ));
        }

        // 1. Fetch the campaign to check its status and target BEFORE donation
        let initial_campaign = self
            .campaign_repo
            .get_campaign(cmd.campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Campaign {} not found", cmd.campaign_id)))?;

        let campaign_name = initial_campaign.name.clone();

        // 2. Check if the campaign is active
        if initial_campaign.status != CampaignStatus::Active {
            return Err(AppError::InvalidOperation(
                "Donations can only be made to active campaigns".to_string(),
            ));
        }

        // 3. Prepare the new donation request
        let req = NewDonationRequest {
            campaign_id: cmd.campaign_id,
            amount: cmd.amount,
            message: cmd.message,
        };

        // 4. Create the donation.
        // The `donation_repo.create` method already handles:
        // - Wallet debit
        // - Donation record insertion
        // - Updating campaign's collected_amount
        // - Logging the transaction
        // All within a single database transaction.
        let donation = self.donation_repo.create(cmd.donor_id, &req).await?;

        // 5. (Potentially Re-fetch or use returned info) Check if campaign target is met
        // After the donation is successfully created, the campaign's collected_amount
        // in the database has been updated by `donation_repo.create`.
        // We need the *new* collected_amount to check against the target.
        // Option A: Re-fetch the campaign (simplest, ensures fresh data)
        let updated_campaign = self
            .campaign_repo
            .get_campaign(cmd.campaign_id)
            .await?
            .ok_or_else(|| {
                // This would be very unusual if the donation succeeded
                AppError::InternalServerError(format!(
                    "Campaign {} not found after successful donation {}",
                    cmd.campaign_id, donation.id
                ))
            })?;

        // Option B: If `donation_repo.create` could also return the new campaign total, use that.
        // For now, re-fetching is safer.

        if updated_campaign.collected_amount >= updated_campaign.target_amount
            && updated_campaign.status == CampaignStatus::Active // Ensure it's still Active before changing
        {
            println!(
                "Campaign {} has reached its target of {}. Collected: {}. Attempting to mark as Completed.",
                updated_campaign.id, updated_campaign.target_amount, updated_campaign.collected_amount
            );
            // This is a separate operation. If it fails, the donation is still valid.
            // Consider logging this failure or putting it in a retry queue.
            match self.campaign_repo
                .update_campaign_status(updated_campaign.id, CampaignStatus::Completed)
                .await
            {
                Ok(true) => {
                    println!("Campaign {} successfully marked as Completed.", updated_campaign.id);
                    let notification_service_clone = Arc::clone(notification_service);

                    tokio::spawn(async move {
                        // Create notification for fundraiser
                        if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                            title: "Campaign Completed".to_string(),
                            content: format!("Your campaign '{}' has reached its target and automatically completed!", 
                                updated_campaign.name),
                            target_type: crate::model::admin::notification::NotificationTargetType::Fundraisers,
                            adt_detail: Some(updated_campaign.id),
                        }).await {
                            eprintln!("Failed to send notification: {:?}", e);
                        }

                        // Create notification for donors
                        if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                            title: "Campaign Target Reached".to_string(),
                            content: format!("The campaign '{}' you donated before has reached its target amount of {}. Thank you for your support!", 
                                updated_campaign.name, updated_campaign.target_amount),
                            target_type: crate::model::admin::notification::NotificationTargetType::Donors,
                            adt_detail: None,
                        }).await {
                            eprintln!("Failed to send notification: {:?}", e);
                        }
                    });
                }
                Ok(false) => {
                     // This means 0 rows were affected, campaign might not exist or status was already Completed
                    eprintln!(
                        "Campaign {} status update to Completed reported no changes. Current status might already be Completed or ID is incorrect.",
                        updated_campaign.id
                    );
                }
                Err(e) => {
                    eprintln!(
                        "ERROR: Donation ID {} created for campaign ID {}, campaign target met, but failed to update campaign status to Completed. Error: {:?}",
                        donation.id, cmd.campaign_id, e
                    );
                    // Don't return an error for the whole donation, as the donation itself succeeded.
                    // This is a secondary effect. Log it and potentially handle it asynchronously.
                }
            }
        }
        
        let notification_service_clone = Arc::clone(notification_service);
        // Create notification for fundraiser
        tokio::spawn(async move {
            if let Err(e) = notification_service_clone.notify(CreateNotificationRequest {
                title: "New Donation Received".to_string(),
                content: format!("A new donation of {} has been made to your campaign '{}'.", 
                    req.amount, campaign_name),
                target_type: crate::model::admin::notification::NotificationTargetType::Fundraisers,
                adt_detail: Some(initial_campaign.id),
            }).await {
                eprintln!("Failed to send notification: {:?}", e);
            }
        });

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

    pub async fn get_total_donated_for_campaign_from_repo( // Renamed to avoid clash if you had another method
        &self,
        campaign_id: i32,
    ) -> Result<i64, AppError> {
        self.donation_repo.get_total_donated_for_campaign(campaign_id).await
    }

    pub async fn get_my_total_for_campaign_from_repo( // Renamed for clarity
        &self,
        user_id: i32,
        campaign_id: i32,
    ) -> Result<i64, AppError> {
        self.donation_repo.get_user_total_for_campaign(user_id, campaign_id).await
    }
}