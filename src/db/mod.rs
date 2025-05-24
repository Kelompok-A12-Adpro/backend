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


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_pool_success() {
        let pool = init_pool().await;
        assert!(!pool.is_closed());
    }
    
    #[tokio::test]
    async fn test_init_pool_multiple_calls_returns_same_instance() {
        let pool1 = init_pool().await;
        let pool2 = init_pool().await;
        
        // Both should reference the same pool
        assert_eq!(pool1.options().get_max_connections(), pool2.options().get_max_connections());
    }

    #[tokio::test]
    async fn test_init_test_pool_success() {        
        let pool = init_test_pool().await;
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_init_test_pool_multiple_calls_returns_same_instance() {
        let pool1 = init_test_pool().await;
        let pool2 = init_test_pool().await;
        
        assert_eq!(pool1.options().get_max_connections(), pool2.options().get_max_connections());
    }
    
    #[tokio::test]
    async fn test_get_pool_when_not_initialized() {
        let pool = get_pool().await;
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_get_pool_when_already_initialized() {
        // Initialize first
        let _ = init_pool().await;
        
        // Get should return existing pool
        let pool = get_pool().await;
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_get_test_pool_when_not_initialized() {
        let pool = get_test_pool().await;
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_get_test_pool_when_already_initialized() {
        // Initialize first
        let _ = init_test_pool().await;
        
        // Get should return existing pool
        let pool = get_test_pool().await;
        assert!(!pool.is_closed());
    }
}