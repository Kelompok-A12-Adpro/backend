use rocket::{State, get, post, put, routes};
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status;
use std::sync::Arc;
use serde::Deserialize;

use crate::model::campaign::campaign::{Campaign, CampaignStatus};
use crate::service::campaign::campaign_service::CampaignService;
use crate::errors::AppError;

#[derive(Deserialize)]
pub struct CreateCampaignRequest {
    pub user_id: i32,
    pub name: String,
    pub description: String,
    pub target_amount: f64,
}

#[post("/campaigns", format = "json", data = "<campaign_req>")]
async fn create_campaign(
    campaign_req: Json<CreateCampaignRequest>,
    service: &State<Arc<CampaignService>>
) -> Result<status::Created<Json<Campaign>>, Status> {
    let result = service.create_campaign(
        campaign_req.user_id,
        campaign_req.name.clone(),
        campaign_req.description.clone(),
        campaign_req.target_amount,
    ).await;
    
    match result {
        Ok(campaign) => {
            let id = campaign.id; // Store ID before moving campaign
            let response = Json(campaign);
            Ok(status::Created::new(format!("/api/campaigns/{}", id)).body(response))
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/campaigns/<campaign_id>")]
async fn get_campaign(
    campaign_id: i32,
    service: &State<Arc<CampaignService>>
) -> Result<Json<Campaign>, Status> {
    match service.get_campaign(campaign_id).await {
        Ok(Some(campaign)) => Ok(Json(campaign)),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/campaigns/user/<user_id>")]
async fn get_user_campaigns(
    user_id: i32,
    service: &State<Arc<CampaignService>>
) -> Result<Json<Vec<Campaign>>, Status> {
    match service.get_campaigns_by_user(user_id).await {
        Ok(campaigns) => Ok(Json(campaigns)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[derive(Deserialize)]
pub struct ApproveCampaignRequest {
    pub admin_id: i32,
}

#[put("/campaigns/<campaign_id>/approve", format = "json", data = "<_approve_req>")]
async fn approve_campaign(
    campaign_id: i32,
    _approve_req: Json<ApproveCampaignRequest>,
    service: &State<Arc<CampaignService>>
) -> Result<Json<Campaign>, Status> {
    match service.approve_campaign(campaign_id).await {
        Ok(campaign) => Ok(Json(campaign)),
        Err(AppError::NotFound(_)) => Err(Status::NotFound),
        Err(AppError::InvalidOperation(_)) => Err(Status::BadRequest),
        Err(_) => Err(Status::InternalServerError),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        create_campaign,
        get_campaign,
        get_user_campaigns,
        approve_campaign
    ]
}