use rocket::{State, post, put, delete, get, routes};
use rocket::serde::json::Json;
use crate::service::campaign_service::CampaignService;
use crate::model::campaign::{Campaign, NewCampaignRequest, UpdateCampaignRequest, EvidenceUploadRequest};
use crate::errors::AppError;
use crate::auth::AuthUser;

/// Create a new campaign
#[post("/campaigns", format = "json", data = "<campaign_req>")]
async fn create_campaign_route(
    auth_user: AuthUser,
    campaign_service: &State<CampaignService>,
    campaign_req: Json<NewCampaignRequest>,
) -> Result<Json<Campaign>, AppError> {
    let campaign = campaign_service
        .create_campaign(auth_user.id, campaign_req.into_inner())
        .await?;
    
    Ok(Json(campaign))
}

/// Get campaign by ID
#[get("/campaigns/<campaign_id>")]
async fn get_campaign_route(
    campaign_service: &State<CampaignService>,
    campaign_id: i32,
) -> Result<Json<Campaign>, AppError> {
    let campaign = campaign_service.get_campaign(campaign_id).await?;
    Ok(Json(campaign))
}

/// Get current user's campaigns
#[get("/campaigns/me")]
async fn get_user_campaigns_route(
    auth_user: AuthUser,
    campaign_service: &State<CampaignService>,
) -> Result<Json<Vec<Campaign>>, AppError> {
    let campaigns = campaign_service.get_user_campaigns(auth_user.id).await?;
    Ok(Json(campaigns))
}

/// Update an existing campaign
#[put("/campaigns/<campaign_id>", format = "json", data = "<campaign_req>")]
async fn update_campaign_route(
    auth_user: AuthUser,
    campaign_service: &State<CampaignService>,
    campaign_id: i32,
    campaign_req: Json<UpdateCampaignRequest>,
) -> Result<Json<Campaign>, AppError> {
    let campaign = campaign_service
        .update_campaign(auth_user.id, campaign_id, campaign_req.into_inner())
        .await?;
    
    Ok(Json(campaign))
}

/// Delete a campaign
#[delete("/campaigns/<campaign_id>")]
async fn delete_campaign_route(
    auth_user: AuthUser,
    campaign_service: &State<CampaignService>,
    campaign_id: i32,
) -> Result<(), AppError> {
    campaign_service.delete_campaign(auth_user.id, campaign_id).await?;
    Ok(())
}

/// Upload evidence for a campaign
#[post("/campaigns/<campaign_id>/evidence", format = "json", data = "<evidence_req>")]
async fn upload_evidence_route(
    auth_user: AuthUser,
    campaign_service: &State<CampaignService>,
    campaign_id: i32,
    evidence_req: Json<EvidenceUploadRequest>,
) -> Result<Json<Campaign>, AppError> {
    let campaign = campaign_service
        .upload_evidence(auth_user.id, campaign_id, evidence_req.evidence_url.clone())
        .await?;
    
    Ok(Json(campaign))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        create_campaign_route,
        get_campaign_route,
        get_user_campaigns_route,
        update_campaign_route,
        delete_campaign_route,
        upload_evidence_route
    ]
}