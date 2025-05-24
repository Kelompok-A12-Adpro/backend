#[macro_use]
extern crate rocket;

use backend::{
    controller::{
        admin::routes::admin_routes,
        donation::routes::donation_routes,
        admin::notification_controller::catchers as notification_catchers,
    },
    state::StateManagement,
    db,
};

#[get("/")]
fn index() -> &'static str {
    "Hello, everynyan!"
}

#[catch(404)]
fn not_found(req: &rocket::Request<'_>) -> String {
    format!("404: '{}' is not a valid route.", req.uri())
}

#[launch]
async fn rocket() -> _ {
    // Initialize environment variables (if using dotenv)
    dotenvy::dotenv().ok(); //dotenvy is newer version of dotenc
    
    // Initialize the database pool singleton
    let pool = db::init_pool().await;
    println!("Database pool initialized.");

    // Initialize all application state
    let app_state = backend::state::init_state(pool).await;
    
    rocket::build()
        .mount("/", routes![index])
        .mount("/admin", admin_routes())
        .mount("/[campaign_id_placeholder]/donation", donation_routes())
        .register("/", catchers![not_found])
        .register("/admin", notification_catchers())
        .manage_state(app_state)
}
