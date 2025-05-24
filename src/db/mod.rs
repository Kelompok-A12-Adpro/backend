use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::sync::OnceLock;
use std::time::Duration;
use std::sync::Once;
static INIT: Once = Once::new();

static DB_POOL: OnceLock<PgPool> = OnceLock::new();
static DB_POOL_TEST: OnceLock<PgPool> = OnceLock::new();

pub async fn init_pool() -> PgPool {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
    });

    // Return the global pool if it's already initialized
    if let Some(pool) = DB_POOL.get() {
        return pool.clone();
    }

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Test the connection to ensure it works
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .expect("Failed to execute test query on database");

    println!("✅ Database connection established successfully");

    // Store in the global static, ignoring errors if it's already set
    let _ = DB_POOL.set(pool.clone());
    
    pool
}

/// Test database pool initialization
pub async fn init_test_pool() -> PgPool {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
    });

    // Return the global pool if it's already initialized
    if let Some(pool) = DB_POOL_TEST.get() {
        return pool.clone();
    }

    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Test the connection to ensure it works
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .expect("Failed to execute test query on database");

    println!("✅ Database connection established successfully");

    // Store in the global static, ignoring errors if it's already set
    let _ = DB_POOL_TEST.set(pool.clone());
    
    pool
}

/// Get the database pool, initializing it if necessary
pub async fn get_pool() -> PgPool {
    if let Some(pool) = DB_POOL.get() {
        pool.clone()
    } else {
        init_pool().await
    }
}

/// Get the test database pool, initializing it if necessary
pub async fn get_test_pool() -> PgPool {
    if let Some(pool) = DB_POOL_TEST.get() {
        pool.clone()
    } else {
        init_test_pool().await
    }
}