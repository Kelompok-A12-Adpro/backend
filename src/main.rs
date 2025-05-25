#[macro_use]
extern crate rocket;

use autometrics::prometheus_exporter;
use backend::{
    controller::{
        admin::{notification_controller::{catchers as notification_catchers, user_routes}, routes::admin_routes}, campaign::routes::campaign_routes, donation::routes::donation_routes
    }, db, state::StateManagement
};

use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

#[get("/")]
fn index() -> &'static str {
    "Hello, everynyan!"
}

#[catch(404)]
fn not_found(req: &rocket::Request<'_>) -> String {
    format!("404: '{}' is not a valid route.", req.uri())
}

#[get("/metrics")]
pub fn metrics() -> String {
    prometheus_exporter::encode_to_string().unwrap()
}

#[launch]
async fn rocket() -> _ {
    // Initialize environment variables (if using dotenv)
    dotenvy::dotenv().ok(); //dotenvy is newer version of dotenc
    
    // CORS Configuration
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec!["Get", "Post", "Put", "Delete"]
                .into_iter()
                .map(|s| s.parse().unwrap())
                .collect(),
        )
        .allowed_headers(AllowedHeaders::all())
        .allow_credentials(true)
        .to_cors()
        .expect("CORS configuration error");

    // Initialize the database pool singleton
    let pool = db::init_pool().await;
    println!("Database pool initialized.");

    // Initialize all application state
    let app_state = backend::state::init_state(pool).await;
    
    rocket::build()
        .mount("/", routes![index])
        .mount("/", campaign_routes())
        .mount("/api/admin", admin_routes())
        .mount("/api", user_routes())
        .mount("/api/donation", donation_routes())
        .register("/", catchers![not_found])
        .register("/admin", notification_catchers())
        .manage_state(app_state)
        .attach(cors)
}
