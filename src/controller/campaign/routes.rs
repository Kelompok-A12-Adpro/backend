use rocket;

pub fn campaign_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    
    routes.extend(super::campaign_controller::routes());

    routes
}
