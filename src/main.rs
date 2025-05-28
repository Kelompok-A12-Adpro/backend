#[macro_use]
extern crate rocket;

use autometrics::prometheus_exporter;
use backend::{
    controller::{
        admin::{notification_controller::{catchers as notification_catchers, user_routes}, routes::admin_routes}, 
        campaign::routes::campaign_routes, 
        donation::routes::donation_routes, 
        wallet::wallet_controller::wallet_routes
    }, 
    db, 
    // Ensure CampaignTotalsCache is accessible, e.g., exported from your donation repository module
    // For example, if it's in `backend::repository::donation_repository`:
    repository::donation_repository::CampaignTotalsCache, // <--- ADD THIS IMPORT (adjust path as needed)
    state::StateManagement // Assuming this is your main state struct
};

use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

// These might be needed if CampaignTotalsCache is defined directly here or for warming
use std::collections::HashMap; // Already needed by CampaignTotalsCache definition
use std::sync::Arc;          // Already needed by CampaignTotalsCache definition
use tokio::sync::Mutex;      // Already needed by CampaignTotalsCache definition


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
    // Initialize environment variables
    dotenvy::dotenv().ok();
    
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

    // --- START CACHE INITIALIZATION ---
    // 1. Create the shared cache instance
    // Assumes CampaignTotalsCache is `Arc<Mutex<HashMap<i32, i64>>>`
    // and is imported from `backend::repository::donation_repository`
    let campaign_totals_cache: CampaignTotalsCache = Arc::new(Mutex::new(HashMap::new()));
    println!("Campaign totals cache created.");

    // 2. Optional: Warm up the cache at startup
    // This queries the database for initial totals and populates the cache.
    // It's good to do this in a non-blocking way if startup time is critical,
    // or ensure it's quick.
    let pool_for_warmup = pool.clone(); // Clone pool for warming task
    let cache_for_warmup = campaign_totals_cache.clone(); // Clone Arc for warming task

    // You can spawn this if you don't want to block rocket's launch,
    // but then the cache might not be warm for the very first requests.
    // For simplicity, doing it inline here:
    println!("Warming up campaign totals cache...");
    match sqlx::query_as::<_, (i32, Option<i64>)>( // (campaign_id, total_amount)
        "SELECT campaign_id, SUM(amount) as total_donated FROM donations GROUP BY campaign_id"
    )
    .fetch_all(&pool_for_warmup)
    .await {
        Ok(campaign_sums) => {
            let mut cache_writer = cache_for_warmup.lock().await;
            for (campaign_id, total_opt) in campaign_sums {
                cache_writer.insert(campaign_id, total_opt.unwrap_or(0));
            }
            println!("Campaign totals cache warmed up with {} entries.", cache_writer.len());
        }
        Err(e) => {
            eprintln!("Failed to warm up campaign totals cache: {}. Continuing without warmed cache.", e);
        }
    }
    // --- END CACHE INITIALIZATION ---


    // Initialize all application state, now passing the cache
    // YOU WILL NEED TO MODIFY `backend::state::init_state` to accept `campaign_totals_cache`
    let app_state = backend::state::init_state(pool.clone(), campaign_totals_cache.clone()).await; // Pass pool and cache
    
    rocket::build()
        .mount("/", routes![index])
        .mount("/", campaign_routes())
        .mount("/api/admin", admin_routes())
        .mount("/api", user_routes())
        .mount("/api/donation", donation_routes())
        .mount("/api", routes![metrics])
        .register("/", catchers![not_found])
        .register("/admin", notification_catchers())
        // .manage_state(app_state) // Your custom state management
        .manage(app_state) // Standard Rocket way to manage state. 
                           // If `manage_state` is a custom macro doing more, adapt as needed.
                           // If `app_state` is an Arc<StateManagement>, you can manage it directly.
                           // Or if StateManagement is cloneable and contains Arcs, it can be managed.
        .attach(cors)
        .mount("/", wallet_routes())
}