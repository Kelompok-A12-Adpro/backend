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
            .mount("/", campaign_controller::routes());
            
        Client::tracked(rocket).expect("valid rocket instance")
    }
    
    #[test]
    fn test_create_campaign() {
        let client = setup();
        
        let response = client.post("/campaigns")
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
        let create_response = client.post("/campaigns")
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
        let get_response = client.get(format!("/campaigns/{}", campaign_id))
            .dispatch();
            
        assert_eq!(get_response.status(), Status::Ok);
        
        let body = get_response.into_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        
        assert_eq!(json["name"], "Test Campaign");
        assert_eq!(json["id"].as_i64().unwrap(), campaign_id);
    }

    #[test]
    fn test_get_all_campaigns() {
        let client = setup();

        // Create first campaign
        client.post("/campaigns")
            .header(ContentType::JSON)
            .body(json!({
                "user_id": 10,
                "name": "First Campaign",
                "description": "First Description",
                "target_amount": 5000.0
            }).to_string())
            .dispatch();

        // Create second campaign
        client.post("/campaigns")
            .header(ContentType::JSON)
            .body(json!({
                "user_id": 20,
                "name": "Second Campaign",
                "description": "Second Description",
                "target_amount": 10000.0
            }).to_string())
            .dispatch();

        // Get all campaigns
        let get_response = client.get("/campaigns")
            .dispatch();
            
        assert_eq!(get_response.status(), Status::Ok);
        
        let body = get_response.into_string().unwrap();
        let campaigns: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
        
        // Verify we got at least 2 campaigns
        assert!(campaigns.len() >= 2);
        
        // Verify our campaigns exist in the response
        let first_campaign = campaigns.iter().find(|c| c["name"] == "First Campaign");
        assert!(first_campaign.is_some());
        let first = first_campaign.unwrap();
        assert_eq!(first["user_id"], 10);
        
        let second_campaign = campaigns.iter().find(|c| c["name"] == "Second Campaign");
        assert!(second_campaign.is_some());
        let second = second_campaign.unwrap();
        assert_eq!(second["user_id"], 20);
    }

    #[test]
    fn test_delete_campaign() {
        let client = setup();

        // create a campaign to delete
        let create = client.post("/campaigns")
            .header(ContentType::JSON)
            .body(json!({
                "user_id": 1,
                "name": "To Be Deleted",
                "description": "desc",
                "target_amount": 100.0
            }).to_string())
            .dispatch();
        assert_eq!(create.status(), Status::Created);
        let body = create.into_string().unwrap();
        let id = serde_json::from_str::<serde_json::Value>(&body).unwrap()["id"].as_i64().unwrap();

        let del = client.delete(format!("/campaigns/{}", id)).dispatch();
        // should be able to delete the campaign
        assert_eq!(del.status(), Status::NoContent);

        let get = client.get(format!("/campaigns/{}", id)).dispatch();
        // should not be able to get the deleted campaign
        assert_eq!(get.status(), Status::NotFound);

        // create a campaign to test delete after approve
        let create2 = client.post("/campaigns")
            .header(ContentType::JSON)
            .body(json!({
                "user_id": 2,
                "name": "Cannot Delete After Approve",
                "description": "desc2",
                "target_amount": 200.0
            }).to_string())
            .dispatch();
        assert_eq!(create2.status(), Status::Created);
        let body2 = create2.into_string().unwrap();
        let id2 = serde_json::from_str::<serde_json::Value>(&body2).unwrap()["id"].as_i64().unwrap();

        // approve the campaign
        let apr = client.put(format!("/campaigns/{}/approve", id2))
            .header(ContentType::JSON)
            .body(json!({ "admin_id": 99 }).to_string())
            .dispatch();
        assert_eq!(apr.status(), Status::Ok);

        let del2 = client.delete(format!("/campaigns/{}", id2)).dispatch();
        // should not be able to delete the campaign after approve
        assert_eq!(del2.status(), Status::BadRequest);

        let get2 = client.get(format!("/campaigns/{}", id2)).dispatch();
        // should be able to get the approved campaign
        assert_eq!(get2.status(), Status::Ok);

    }
}