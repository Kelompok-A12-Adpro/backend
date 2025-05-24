use chrono::{DateTime, Utc};
use rocket::{State, get, post, put, delete, routes, uri};
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::status::Created;
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
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCampaignRequest {
    pub user_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_amount: Option<f64>,
    pub end_date: Option<DateTime<Utc>>,
    pub image_url: Option<String>,

}

#[post("/campaigns", format = "json", data = "<campaign_req>")]
async fn create_campaign(
    campaign_req: Json<CreateCampaignRequest>,
    service: &State<Arc<CampaignService>>
) -> Result<Created<Json<Campaign>>, Status> {
    let campaign = service.create_campaign(
        campaign_req.user_id,
        campaign_req.name.clone(),
        campaign_req.description.clone(),
        campaign_req.target_amount,
        campaign_req.start_date,
        campaign_req.end_date,
        campaign_req.image_url.clone(),
    ).await.map_err(|_| Status::InternalServerError)?;

    // generate the correct URI, including any mount‐point (e.g. "/api")
    let location = uri!(get_campaign(campaign.id)).to_string();
    Ok(Created::new(location).body(Json(campaign)))
}

#[put("/campaigns/<id>", format = "json", data = "<update_req>")]
async fn update_campaign(
    id: i32,
    update_req: Json<UpdateCampaignRequest>,
    service: &State<Arc<CampaignService>>
) -> Result<Json<Campaign>, Status> {
    
    match service.update_campaign(
        id,
        update_req.user_id,
        update_req.name.clone(),
        update_req.description.clone(),
        update_req.target_amount,
        update_req.end_date,
        update_req.image_url.clone(),
    ).await {
        Ok(campaign) => Ok(Json(campaign)),
        Err(AppError::NotFound(_)) => Err(Status::NotFound),
        Err(AppError::InvalidOperation(_)) => Err(Status::BadRequest),
        Err(_) => Err(Status::InternalServerError),
    }
}


#[get("/campaigns")]
async fn get_all_campaigns(
    service: &State<Arc<CampaignService>>
) -> Result<Json<Vec<Campaign>>, Status> {
    match service.get_all_campaigns().await {
        Ok(campaigns) => Ok(Json(campaigns)),
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

#[delete("/campaigns/<campaign_id>")]
async fn delete_campaign(
    campaign_id: i32,
    service: &State<Arc<CampaignService>>
) -> Result<Status, Status> {
    match service.delete_campaign(campaign_id).await {
        Ok(_) => Ok(Status::NoContent),
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
        approve_campaign,
        get_all_campaigns,
        delete_campaign,
        update_campaign,
    ]
}