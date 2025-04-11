use rocket::{State, get, put, routes};
use rocket::serde::json::Json;
use crate::model::admin::campaign_admin::{AdminCampaignSummary, CampaignStatus, CampaignVerificationRequest};

// Placeholder for simplified controllers
#[get("/admin/campaigns")]
fn get_campaigns() -> Json<Vec<AdminCampaignSummary>> {
    Json(Vec::new())
}

#[get("/admin/campaigns/<_campaign_id>")]
fn get_campaign_details(_campaign_id: i32) -> Json<AdminCampaignSummary> {
    Json(AdminCampaignSummary {
        id: 0,
        title: String::new(),
        fundraiser_id: 0,
        fundraiser_name: String::new(),
        start_date: chrono::Utc::now(),
        end_date: chrono::Utc::now(),
        target_amount: 0.0,
        collected_amount: 0.0,
        status: CampaignStatus::PendingVerification,
    })
}

#[put("/admin/campaigns/<_campaign_id>/verify", format = "json", data = "<_verification_req>")]
fn verify_campaign(_campaign_id: i32, _verification_req: Json<CampaignVerificationRequest>) -> Json<AdminCampaignSummary> {
    Json(AdminCampaignSummary {
        id: 0,
        title: String::new(),
        fundraiser_id: 0,
        fundraiser_name: String::new(),
        start_date: chrono::Utc::now(),
        end_date: chrono::Utc::now(),
        target_amount: 0.0,
        collected_amount: 0.0,
        status: CampaignStatus::PendingVerification,
    })
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_campaigns,
        get_campaign_details,
        verify_campaign
    ]
}
