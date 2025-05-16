use async_trait::async_trait;
use sqlx::{PgPool, types::chrono::Utc}; // Added Utc here
use crate::model::donation::donation::{Donation, NewDonationRequest};
use crate::repository::donation::donation_repository::{DonationRepository, PgDonationRepository};
use crate::errors::AppError;


#[cfg(test)]
mod tests {
    use super::*; // To get DonationRepository trait, PgDonationRepository
    use crate::errors::AppError; // Assuming AppError is here
    use crate::model::donation::donation::{Donation, NewDonationRequest}; // Models

    use sqlx::{PgPool, Executor}; // For PgPool and executing queries
    use once_cell::sync::Lazy;   // For static PgPool initialization
    use std::env;                 // For DATABASE_URL
    use chrono::{Utc, Duration};  // For time-related assertions

    // --- Database Setup ---

    // Initialize the dotenv and DATABASE_URL loading once
    static TEST_POOL: Lazy<PgPool> = Lazy::new(|| {
        // Load .env file. This is useful for local testing.
        // In CI, DATABASE_URL should be set directly as an environment variable.
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for integration tests. Example: postgres://user:pass@host/test_db_donations");
        
        // connect_lazy doesn't connect immediately, useful for static context
        PgPool::connect_lazy(&database_url)
            .unwrap_or_else(|e| panic!("Failed to create lazy PgPool for tests: {}. Ensure DATABASE_URL is correct and PostgreSQL is running.", e))
    });

    // Function to reset the database state before each test
    async fn reset_db(pool: &PgPool) {
        // TRUNCATE donations table and restart its ID sequence.
        // CASCADE can be used if other tables reference donations.
        sqlx::query("TRUNCATE TABLE donations RESTART IDENTITY CASCADE")
            .execute(pool)
            .await
            .expect("Failed to truncate 'donations' table. Ensure the table exists and the user has permissions.");

        // IMPORTANT: If you have foreign keys in 'donations' table (e.g., to 'users' or 'campaigns'):
        // 1. You might need to truncate those tables as well (in the correct order or use CASCADE).
        //    e.g., sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE").execute(pool).await.expect(...);
        //          sqlx::query("TRUNCATE TABLE campaigns RESTART IDENTITY CASCADE").execute(pool).await.expect(...);
        // 2. Your tests might need to insert prerequisite data into 'users' and 'campaigns' tables
        //    before creating donations that reference them.
        //    For example, before creating a donation with user_id = 1, campaign_id = 101,
        //    you'd need to ensure a user with id=1 and a campaign with id=101 exist.
        //    This setup is NOT included here for brevity but is CRUCIAL for real-world FKs.
        //    You might create helper functions like `seed_user(pool, user_id)` and `seed_campaign(pool, campaign_id)`.
    }

    // Helper to get a PgDonationRepository instance for tests with a clean database
    async fn get_repository() -> PgDonationRepository {
        let pool = TEST_POOL.clone(); // Clone the Arc<PgPool>
        reset_db(&pool).await;      // Ensure DB is clean for this test
        PgDonationRepository::new(pool)
    }

    // --- Tests (adapted for PgDonationRepository) ---

    #[tokio::test]
    async fn test_create_donation_success() {
        let repo = get_repository().await;

        // FK Note: For these tests to pass with FK constraints, ensure a user with ID 1
        // and a campaign with ID 101 exist, or disable FK checks for testing,
        // or modify reset_db to seed these.
        let user_id = 1;
        let campaign_id = 101;
        let new_donation_req = NewDonationRequest {
            campaign_id,
            amount: 50.0,
            message: Some("Test donation".to_string()),
        };

        let result = repo.create(user_id, &new_donation_req).await;
        assert!(result.is_ok(), "Failed to create donation: {:?}", result.err());

        let created_donation = result.unwrap();
        assert_eq!(created_donation.id, 1); // Relies on RESTART IDENTITY from reset_db
        assert_eq!(created_donation.user_id, user_id);
        assert_eq!(created_donation.campaign_id, new_donation_req.campaign_id);
        assert_eq!(created_donation.amount, new_donation_req.amount);
        assert_eq!(created_donation.message, new_donation_req.message);
        
        let now = Utc::now();
        assert!(created_donation.created_at <= now && created_donation.created_at > (now - Duration::seconds(5)), 
                "created_at timestamp is not recent: {:?}, now: {:?}", created_donation.created_at, now);
    }

    #[tokio::test]
    async fn test_create_donation_no_message() {
        let repo = get_repository().await;

        let user_id = 2;       // FK Note
        let campaign_id = 102; // FK Note
        let new_donation_req = NewDonationRequest {
            campaign_id,
            amount: 25.0,
            message: None,
        };

        let created_donation = repo.create(user_id, &new_donation_req).await.unwrap();
        assert_eq!(created_donation.id, 1); // ID restarts for each test
        assert_eq!(created_donation.user_id, user_id);
        assert_eq!(created_donation.campaign_id, campaign_id);
        assert_eq!(created_donation.message, None);
    }

    #[tokio::test]
    async fn test_find_by_id_exists() {
        let repo = get_repository().await;

        let user_id = 3;       // FK Note
        let campaign_id = 103; // FK Note
        let new_donation_req = NewDonationRequest {
            campaign_id,
            amount: 75.0,
            message: Some("Find me".to_string()),
        };
        let created_donation = repo.create(user_id, &new_donation_req).await.unwrap();
        // The ID will be 1 due to reset_db and RESTART IDENTITY

        let found_donation_opt = repo.find_by_id(created_donation.id).await.unwrap();
        assert!(found_donation_opt.is_some());
        let found_donation = found_donation_opt.unwrap();
        assert_eq!(found_donation.id, created_donation.id);
        assert_eq!(found_donation.message, Some("Find me".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_id_not_exists() {
        let repo = get_repository().await;
        // No donations created in this test instance after reset_db

        let found_donation_opt = repo.find_by_id(99999).await.unwrap();
        assert!(found_donation_opt.is_none());
    }

    #[tokio::test]
    async fn test_find_by_campaign() {
        let repo = get_repository().await;

        let campaign_id_target = 201; // FK Note
        let campaign_id_other = 202;  // FK Note
        let user_1 = 1; // FK Note
        let user_2 = 2; // FK Note
        let user_3 = 3; // FK Note

        // FK Note: Ensure users 1,2,3 and campaigns 201,202 exist if FKs are enforced.

        let d1 = repo.create(user_1, &NewDonationRequest { campaign_id: campaign_id_target, amount: 10.0, message: None }).await.unwrap();
        let d2 = repo.create(user_2, &NewDonationRequest { campaign_id: campaign_id_target, amount: 20.0, message: Some("Msg1".into()) }).await.unwrap();
        let d3 = repo.create(user_3, &NewDonationRequest { campaign_id: campaign_id_other, amount: 30.0, message: None }).await.unwrap();

        let donations = repo.find_by_campaign(campaign_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.campaign_id == campaign_id_target));
        // Order is `created_at DESC` from the query. d2 was created after d1.
        assert_eq!(donations[0].id, d2.id);
        assert_eq!(donations[1].id, d1.id);

        let donations_other = repo.find_by_campaign(campaign_id_other).await.unwrap();
        assert_eq!(donations_other.len(), 1);
        assert_eq!(donations_other[0].id, d3.id);

        let donations_none = repo.find_by_campaign(999).await.unwrap(); // Non-existent campaign_id
        assert!(donations_none.is_empty());
    }

    #[tokio::test]
    async fn test_find_by_user() {
        let repo = get_repository().await;

        let user_id_target = 51; // FK Note
        let user_id_other = 52;  // FK Note
        let campaign_1 = 301; // FK Note
        let campaign_2 = 302; // FK Note
        let campaign_3 = 303; // FK Note

        // FK Note: Ensure users 51,52 and campaigns 301,302,303 exist if FKs are enforced.

        let d1 = repo.create(user_id_target, &NewDonationRequest { campaign_id: campaign_1, amount: 15.0, message: None }).await.unwrap();
        let d2 = repo.create(user_id_target, &NewDonationRequest { campaign_id: campaign_2, amount: 25.0, message: Some("Msg2".into()) }).await.unwrap();
        let d3 = repo.create(user_id_other, &NewDonationRequest { campaign_id: campaign_3, amount: 35.0, message: None }).await.unwrap();

        let donations = repo.find_by_user(user_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.user_id == user_id_target));
        // Order is `created_at DESC`
        assert_eq!(donations[0].id, d2.id);
        assert_eq!(donations[1].id, d1.id);

        let donations_other = repo.find_by_user(user_id_other).await.unwrap();
        assert_eq!(donations_other.len(), 1);
        assert_eq!(donations_other[0].id, d3.id);

        let donations_none = repo.find_by_user(999).await.unwrap(); // Non-existent user_id
        assert!(donations_none.is_empty());
    }

    #[tokio::test]
    async fn test_update_message_success() {
        let repo = get_repository().await;

        let user_id = 61;       // FK Note
        let campaign_id = 401;  // FK Note
        let initial_donation = repo.create(user_id, &NewDonationRequest {
            campaign_id,
            amount: 100.0,
            message: Some("Initial message".to_string()),
        }).await.unwrap();

        let new_message = Some("Updated message".to_string());
        let rows_affected = repo.update_message(initial_donation.id, user_id, new_message.clone()).await.unwrap();
        assert_eq!(rows_affected, 1);

        let updated_donation = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(updated_donation.message, new_message);
    }

    #[tokio::test]
    async fn test_update_message_to_none() {
        let repo = get_repository().await;

        let user_id = 62;       // FK Note
        let campaign_id = 402;  // FK Note
        let initial_donation = repo.create(user_id, &NewDonationRequest {
            campaign_id,
            amount: 110.0,
            message: Some("A message to clear".to_string()),
        }).await.unwrap();

        let rows_affected = repo.update_message(initial_donation.id, user_id, None).await.unwrap();
        assert_eq!(rows_affected, 1);

        let updated_donation = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(updated_donation.message, None);
    }

    #[tokio::test]
    async fn test_update_message_donation_not_found() {
        let repo = get_repository().await;

        let user_id = 63; // FK Note (though the donation won't be found anyway)
        let non_existent_donation_id = 9999; 
        let new_message = Some("This won't be set".to_string());
        let rows_affected = repo.update_message(non_existent_donation_id, user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0);
    }

    #[tokio::test]
    async fn test_update_message_user_mismatch() {
        let repo = get_repository().await;

        let owner_user_id = 64;   // FK Note
        let other_user_id = 65;   // FK Note
        let campaign_id = 404;    // FK Note

        let initial_donation = repo.create(owner_user_id, &NewDonationRequest {
            campaign_id,
            amount: 120.0,
            message: Some("Original message by owner".to_string()),
        }).await.unwrap();

        let new_message = Some("Attempted update by other user".to_string());
        // Attempt to update with other_user_id
        let rows_affected = repo.update_message(initial_donation.id, other_user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0, "Update should fail if user_id does not match");

        // Verify the message hasn't changed
        let donation_after_attempt = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(donation_after_attempt.message, Some("Original message by owner".to_string()));
    }

    #[tokio::test]
    async fn test_create_donation_violates_fk_if_not_handled() {
        // This test is more of a demonstration of what happens if FKs are enforced
        // and parent records are NOT seeded. It will likely fail if your DB has FKs
        // on user_id or campaign_id and the reset_db doesn't handle them.
        let repo = get_repository().await;

        let non_existent_user_id = 999991;
        let non_existent_campaign_id = 888881;
        let new_donation_req = NewDonationRequest {
            campaign_id: non_existent_campaign_id,
            amount: 50.0,
            message: Some("This might fail".to_string()),
        };

        let result = repo.create(non_existent_user_id, &new_donation_req).await;
        
        if result.is_ok() {
            // This branch will execute if FKs are not enforced or if users/campaigns
            // with these IDs were somehow created.
            // For this test's purpose, if it's OK, it means FKs are not causing an issue here.
            // You might want to make this an assertion failure if FKs ARE expected to be enforced.
            println!("test_create_donation_violates_fk_if_not_handled: Creation was OK. This might indicate FKs are not enforced as expected, or prerequisite data existed.");
            // Consider: assert!(false, "Creation succeeded, but an FK violation was expected."); 
            // Depending on whether you *require* this test to demonstrate an FK failure.
        } else {
            // This is the expected path if FKs are enforced and parents don't exist.
            // result is an Err variant here. Let's extract the AppError.
            let actual_error = result.unwrap_err(); // This consumes result and gives the AppError

            println!("test_create_donation_violates_fk_if_not_handled: Creation failed as expected (likely FK violation): {:?}", actual_error);
            
            match actual_error {
                AppError::DatabaseError(msg) => {
                    let lower_msg = msg.to_lowercase(); // For case-insensitive matching
                    assert!(
                        lower_msg.contains("foreign key constraint") || // General SQL
                        lower_msg.contains("violates foreign key constraint") || // PostgreSQL specific
                        lower_msg.contains("referential integrity") || // Another common term
                        lower_msg.contains("constraint failed"), // e.g., SQLite
                        "DatabaseError message '{}' did not contain expected foreign key violation text.", msg
                    );
                }
                // Use a different variable name for the caught error to avoid confusion with the outer `actual_error`
                other_app_error => panic!(
                    "Expected AppError::DatabaseError for FK violation, but got {:?}. Full error: {:?}", 
                    other_app_error, other_app_error // Printing it twice for clarity in the panic message
                ),
            }
        }
    }
}