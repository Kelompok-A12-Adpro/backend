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
    repository::donation::donation_repository::CampaignTotalsCache,
    repository::donation::donation_repository::UserCampaignDonationCache,
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
    let campaign_totals_cache: CampaignTotalsCache = Arc::new(Mutex::new(HashMap::new()));
    println!("Campaign totals cache (global) created.");

    let user_campaign_donation_cache: UserCampaignDonationCache = Arc::new(Mutex::new(HashMap::new()));
    println!("User campaign donation cache (per-user) created.");

    // 3. Optional: Warm up caches
    let pool_for_warmup = pool.clone();
    let global_cache_for_warmup = campaign_totals_cache.clone();
    let user_cache_for_warmup = user_campaign_donation_cache.clone();

    // Warm up global campaign totals
    tokio::spawn(async move { // Spawn warming to not block startup, or do it inline if preferred
        println!("Warming up global campaign totals cache...");
        match sqlx::query_as::<_, (i32, Option<i64>)>(
            "SELECT campaign_id, SUM(amount) as total_donated FROM donations GROUP BY campaign_id"
        )
        .fetch_all(&pool_for_warmup)
        .await {
            Ok(campaign_sums) => {
                let mut cache_writer = global_cache_for_warmup.lock().await;
                for (campaign_id, total_opt) in campaign_sums {
                    cache_writer.insert(campaign_id, total_opt.unwrap_or(0));
                }
                println!("Global campaign totals cache warmed with {} entries.", cache_writer.len());
            }
            Err(e) => eprintln!("Failed to warm up global campaign totals cache: {}", e),
        }
    });
    
    // Warm up user-specific campaign totals
    let pool_for_user_warmup = pool.clone(); // Need another clone if previous task took ownership
    tokio::spawn(async move {
        println!("Warming up user-specific campaign donation cache...");
        // This query might return many rows if you have many users and donations
        match sqlx::query_as::<_, (i32, i32, Option<i64>)>( // (user_id, campaign_id, total_for_pair)
            "SELECT user_id, campaign_id, SUM(amount) as total_donated FROM donations GROUP BY user_id, campaign_id"
        )
        .fetch_all(&pool_for_user_warmup) // Use the new cloned pool
        .await {
            Ok(user_campaign_sums) => {
                let mut cache_writer = user_cache_for_warmup.lock().await;
                for (user_id, campaign_id, total_opt) in user_campaign_sums {
                    cache_writer
                        .entry(user_id)
                        .or_default()
                        .insert(campaign_id, total_opt.unwrap_or(0));
                }
                println!("User-specific campaign donation cache warmed with data for {} users.", cache_writer.len());
            }
            Err(e) => eprintln!("Failed to warm up user-specific campaign donation cache: {}", e),
        }
    });
    // --- END CACHE INITIALIZATION ---


    // Initialize all application state, now passing the cache
    // YOU WILL NEED TO MODIFY `backend::state::init_state` to accept `campaign_totals_cache`
    let app_state = backend::state::init_state(
        pool.clone(), 
        campaign_totals_cache.clone(), 
        user_campaign_donation_cache.clone()).await; // Pass pool and cache
    
    rocket::build()
        .mount("/", routes![index])
        .mount("/", campaign_routes())
        .mount("/api/admin", admin_routes())
        .mount("/api", user_routes())
        .mount("/api/donation", donation_routes())
        .mount("/api", routes![metrics])
        .register("/", catchers![not_found])
        .register("/api/admin", notification_catchers())
        .manage_state(app_state) // Your custom state management
        .attach(cors)
        .mount("/", wallet_routes())
}