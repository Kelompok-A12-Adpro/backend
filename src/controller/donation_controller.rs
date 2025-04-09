use rocket::{State, post, delete, get, routes};
use rocket::serde::json::Json;
use crate::service::donation_service::DonationService;
use crate::model::donation::{NewDonationRequest, Donation};
use crate::errors::AppError;
use crate::auth::AuthUser; 


#[post("/donations", format = "json", data = "<donation_req>")]
async fn make_donation_route(
    auth_user: AuthUser, 
    donation_service: &State<DonationService>,
    donation_req: Json<NewDonationRequest>,
) -> Result<Json<Donation>, AppError> {
    let cmd = crate::service::commands::donation_commands::MakeDonationCommand {
        donor_id: auth_user.id,
        campaign_id: donation_req.campaign_id,
        amount: donation_req.amount,
        message: donation_req.message.clone(),
    };
    let donation = donation_service.make_donation(cmd).await?;
    Ok(Json(donation))
}


#[delete("/donations/<donation_id>/message")]
async fn delete_donation_message_route(
    auth_user: AuthUser,
    donation_service: &State<DonationService>,
    donation_id: i32,
) -> Result<(), AppError> { 
    let cmd = crate::service::commands::donation_commands::DeleteDonationMessageCommand {
        donation_id,
        user_id: auth_user.id,
    };
    donation_service.delete_donation_message(cmd).await?;
    Ok(())
}


#[get("/campaigns/<campaign_id>/donations")]
async fn get_campaign_donations_route(
    donation_service: &State<DonationService>,
    campaign_id: i32,
    
) -> Result<Json<Vec<Donation>>, AppError> {
    let donations = donation_service.get_donations_by_campaign(campaign_id).await?;
    Ok(Json(donations))
}


#[get("/donations/me")]
async fn get_my_donations_route(
    auth_user: AuthUser,
    donation_service: &State<DonationService>,
) -> Result<Json<Vec<Donation>>, AppError> {
    let donations = donation_service.get_donations_by_user(auth_user.id).await?;
    Ok(Json(donations))
}


pub fn routes() -> Vec<rocket::Route> {
    routes![
        make_donation_route,
        delete_donation_message_route,
        get_campaign_donations_route,
        get_my_donations_route
    ]
}
