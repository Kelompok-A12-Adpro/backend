use rocket::{get, put, routes};
use rocket::serde::json::Json;
use std::collections::HashMap;

// Placeholder for simplified controllers
#[get("/campaigns")]
fn get_campaigns() -> Json<Vec<HashMap<String, String>>> {
    Json(Vec::new())
}

#[get("/campaigns/<_campaign_id>")]
fn get_campaign_details(_campaign_id: i32) -> Json<HashMap<String, String>> {
    Json(HashMap::new())
}

#[put("/campaigns/<_campaign_id>/verify", format = "json", data = "<_verification_req>")]
fn verify_campaign(_campaign_id: i32, _verification_req: Json<HashMap<String, String>>) -> Json<HashMap<String, String>> {
    Json(HashMap::new())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_campaigns,
        get_campaign_details,
        verify_campaign
    ]
}
