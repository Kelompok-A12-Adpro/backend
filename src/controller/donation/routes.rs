use rocket;

pub fn donation_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    
    routes.extend(super::donation_controller::routes());
    
    routes
}
