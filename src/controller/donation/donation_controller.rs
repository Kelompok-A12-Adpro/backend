use std::sync::Arc;

use autometrics::autometrics;
use rocket::{State, post, delete, get, routes, response::status};
use rocket::serde::json::Json;
use serde::Serialize;

use crate::service::admin::notification::notification_service::NotificationService;
use crate::service::donation::donation_service::DonationService;

use crate::model::donation::donation::{NewDonationRequest, Donation};
use crate::errors::AppError;
use crate::controller::auth::auth::AuthUser;

#[derive(Serialize, Debug)]
pub struct TotalAmountResponse {
    pub total_amount: i64,
}

#[post("/donations", format = "json", data = "<donation_req>")]
#[autometrics]
async fn make_donation_route(
    auth_user: AuthUser,
    donation_service: &State<DonationService>, // Or &State<Arc<dyn DonationServiceTrait>>
    donation_req: Json<NewDonationRequest>,
    notification_service: &State<Arc<NotificationService>>,
) -> Result<status::Created<Json<Donation>>, AppError> { // Changed to return 201 Created

    let cmd = crate::service::commands::donation_commands::MakeDonationCommand {
        donor_id: auth_user.id, // Use the ID from the dummy user
        campaign_id: donation_req.campaign_id,
        amount: donation_req.amount,
        message: donation_req.message.clone(),
    };
    let donation = donation_service.make_donation(cmd, notification_service).await?;
    let location = format!("/api/donations/{}", donation.id); // Construct location for 201
    Ok(status::Created::new(location).body(Json(donation)))
}


#[delete("/donations/<donation_id>/message")]
#[autometrics]
async fn delete_donation_message_route(
    auth_user: AuthUser,
    donation_service: &State<DonationService>, // Or &State<Arc<dyn DonationServiceTrait>>
    donation_id: i32,
) -> Result<status::NoContent, AppError> { // Changed to return 204 No Content

    let cmd = crate::service::commands::donation_commands::DeleteDonationMessageCommand {
        donation_id,
        user_id: auth_user.id, // Use the ID from the dummy user
    };
    donation_service.delete_donation_message(cmd).await?;
    Ok(status::NoContent)
}


#[get("/campaigns/<campaign_id>/donations")]
#[autometrics]
async fn get_campaign_donations_route(
    donation_service: &State<DonationService>,
    campaign_id: i32,
) -> Result<Json<Vec<Donation>>, AppError> {
    let donations = donation_service.get_donations_by_campaign(campaign_id).await?;
    Ok(Json(donations))
}


#[get("/donations/me")]
#[autometrics]
async fn get_my_donations_route(
    auth_user: AuthUser,
    donation_service: &State<DonationService>, // Or &State<Arc<dyn DonationServiceTrait>>
) -> Result<Json<Vec<Donation>>, AppError> {

    let donations = donation_service.get_donations_by_user(auth_user.id).await?;
    Ok(Json(donations))
}

#[get("/campaigns/<campaign_id>/donations/total")]
#[autometrics]
async fn get_campaign_total_donations_route(
    donation_service: &State<DonationService>,
    campaign_id: i32,
) -> Result<Json<TotalAmountResponse>, AppError> {
    let total = donation_service.get_total_donated_for_campaign_from_repo(campaign_id).await?;
    Ok(Json(TotalAmountResponse { total_amount: total }))
}

#[get("/donations/me/campaigns/<campaign_id>/total")]
#[autometrics]
async fn get_my_total_donations_for_campaign_route(
    auth_user: AuthUser,
    donation_service: &State<DonationService>,
    campaign_id: i32,
) -> Result<Json<TotalAmountResponse>, AppError> {
    let total = donation_service.get_my_total_for_campaign_from_repo(auth_user.id, campaign_id).await?;
    Ok(Json(TotalAmountResponse { total_amount: total }))
}


pub fn routes() -> Vec<rocket::Route> {
    routes![
        make_donation_route,
        delete_donation_message_route,
        get_campaign_donations_route,
        get_my_donations_route,
        get_campaign_total_donations_route,
        get_my_total_donations_for_campaign_route
    ]
}