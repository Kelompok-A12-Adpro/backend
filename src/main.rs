#[macro_use]
extern crate rocket;

use std::sync::Arc;
use backend::controller::campaign::campaign_controller;
use backend::service::campaign::campaign_service::CampaignService;
use backend::service::campaign::factory::campaign_factory::CampaignFactory;
use backend::repository::campaign::campaign_repository::InMemoryCampaignRepository;
use backend::service::campaign::observer::campaign_observer::CampaignNotifier;


#[get("/")]
fn index() -> &'static str {
    "Hello, everynyan!"
}

#[get("/<name>")]
fn name(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[catch(404)]
fn not_found(req: &rocket::Request<'_>) -> String {
    format!("Sorry, '{}' is not a valid route..", req.uri())
}

#[launch]
fn rocket() -> _ {

    // Temporary in-memory repository for testing purposes
    let repo = Arc::new(InMemoryCampaignRepository::new());
    let factory = Arc::new(CampaignFactory::new());
    let notifier = Arc::new(CampaignNotifier::new());
    let campaign_service = Arc::new(CampaignService::new(repo, factory, notifier));

    rocket::build()
        .mount("/", routes![index, name])
        .mount("/api", campaign_controller::routes())
        .manage(campaign_service)
        .register("/", catchers![not_found])
}
