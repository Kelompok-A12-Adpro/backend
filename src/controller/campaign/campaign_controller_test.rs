#[cfg(test)]
mod tests {
    use rocket::local::blocking::Client;
    use rocket::http::{ContentType, Status};
    use crate::controller::campaign::campaign_controller;
    use crate::service::campaign::campaign_service::CampaignService;
    use crate::service::campaign::factory::campaign_factory::CampaignFactory;
    use crate::repository::campaign::campaign_repository::InMemoryCampaignRepository;
    use crate::service::campaign::observer::campaign_observer::CampaignNotifier;
    use std::sync::Arc;
    use serde_json::json;

    fn setup() -> Client {
        let repo = Arc::new(InMemoryCampaignRepository::new());
        let factory = Arc::new(CampaignFactory::new());
        let notifier = Arc::new(CampaignNotifier::new());
        let service = Arc::new(CampaignService::new(repo, factory, notifier));
        
        let rocket = rocket::build()
            .manage(service)
            .mount("/api", campaign_controller::routes());
            
        Client::tracked(rocket).expect("valid rocket instance")
    }
    
    #[test]
    fn test_create_campaign() {
        let client = setup();
        
        let response = client.post("/api/campaigns")
            .header(ContentType::JSON)
            .body(json!({
                "user_id": 10,
                "name": "Test Campaign",
                "description": "Test Description",
                "target_amount": 5000.0
            }).to_string())
            .dispatch();
            
        assert_eq!(response.status(), Status::Created);
        
        let body = response.into_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        
        assert_eq!(json["name"], "Test Campaign");
        assert_eq!(json["user_id"], 10);
        assert_eq!(json["target_amount"], 5000.0);
    }
    
    #[test]
    fn test_get_campaign() {
        let client = setup();
        
        // First, create a campaign
        let create_response = client.post("/api/campaigns")
            .header(ContentType::JSON)
            .body(json!({
                "user_id": 10,
                "name": "Test Campaign",
                "description": "Test Description",
                "target_amount": 5000.0
            }).to_string())
            .dispatch();
            
        let body = create_response.into_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let campaign_id = json["id"].as_i64().unwrap();
        
        // Then, get it
        let get_response = client.get(format!("/api/campaigns/{}", campaign_id))
            .dispatch();
            
        assert_eq!(get_response.status(), Status::Ok);
        
        let body = get_response.into_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        
        assert_eq!(json["name"], "Test Campaign");
        assert_eq!(json["id"].as_i64().unwrap(), campaign_id);
    }
}