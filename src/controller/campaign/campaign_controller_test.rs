#[cfg(test)]
mod tests {
    use rocket::local::blocking::Client;
    use rocket::http::{ContentType, Status};
    use rocket::{State, post, get, routes};
    use rocket::serde::json::Json;
    use std::sync::Arc;
    use serde_json::json;
    use chrono::Utc;

    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use crate::errors::AppError;

    struct MockCampaignService;

    impl MockCampaignService {
        fn new() -> Self { MockCampaignService }

        async fn create_campaign(
            &self,
            user_id: i32,
            name: String,
            description: String,
            target_amount: i64,
            start_date: chrono::DateTime<chrono::Utc>,
            end_date: chrono::DateTime<chrono::Utc>,
            image_url: Option<String>,
        ) -> Result<Campaign, AppError> {
            let now = Utc::now();
            Ok(Campaign {
                id: 1,
                user_id,
                name,
                description,
                target_amount,
                collected_amount: 0,
                start_date,
                end_date,
                image_url,
                status: CampaignStatus::PendingVerification,
                created_at: now,
                updated_at: now,
                evidence_url: None,
                evidence_uploaded_at: None,
            })
        }

        async fn get_campaign(&self, id: i32) -> Result<Option<Campaign>, AppError> {
            if id == 1 {
                let now = Utc::now();
                Ok(Some(Campaign {
                    id,
                    user_id: 1,
                    name: "Test Campaign".to_string(),
                    description: "Test Description".to_string(),
                    target_amount: 5000,
                    collected_amount: 0,
                    start_date: now,
                    end_date: now + chrono::Duration::days(30),
                    image_url: None,
                    status: CampaignStatus::PendingVerification,
                    created_at: now,
                    updated_at: now,
                    evidence_url: None,
                    evidence_uploaded_at: None,
                }))
            } else {
                Ok(None)
            }
        }
    }

    #[post("/campaigns", format = "json", data = "<req>")]
    async fn create_campaign_mock(
        req: Json<serde_json::Value>,
        service: &State<Arc<MockCampaignService>>
    ) -> Result<Json<Campaign>, Status> {
        let result = service.create_campaign(
            1,
            req["name"].as_str().unwrap().to_string(),
            req["description"].as_str().unwrap().to_string(),
            req["target_amount"].as_i64().unwrap(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
            None,
        ).await;

        match result {
            Ok(campaign) => Ok(Json(campaign)),
            Err(_) => Err(Status::InternalServerError),
        }
    }

    #[get("/campaigns/<id>")]
    async fn get_campaign_mock(
        id: i32,
        service: &State<Arc<MockCampaignService>>
    ) -> Result<Json<Campaign>, Status> {
        match service.get_campaign(id).await {
            Ok(Some(campaign)) => Ok(Json(campaign)),
            Ok(None) => Err(Status::NotFound),
            Err(_) => Err(Status::InternalServerError),
        }
    }

    fn setup() -> Client {
        let mock_service = Arc::new(MockCampaignService::new());
        
        let rocket = rocket::build()
            .manage(mock_service)
            .mount("/", routes![create_campaign_mock, get_campaign_mock]);
            
        Client::tracked(rocket).expect("valid rocket instance")
    }

    #[test]
    fn test_create_campaign() {
        let client = setup();
        
        let response = client.post("/campaigns")
            .header(ContentType::JSON)
            .body(json!({
                "name": "Test Campaign",
                "description": "Test Description",
                "target_amount": 5000
            }).to_string())
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_get_campaign() {
        let client = setup();
        
        let response = client.get("/campaigns/1").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_get_campaign_not_found() {
        let client = setup();
        
        let response = client.get("/campaigns/999").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }
}