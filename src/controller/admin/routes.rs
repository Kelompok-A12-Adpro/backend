use rocket;

pub fn admin_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    
    routes.extend(super::dashboard_controller::routes());
    routes.extend(super::notification_controller::admin_routes());
    
    routes
}

pub fn user_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    
    routes.extend(super::notification_controller::user_routes());
    
    routes
}
