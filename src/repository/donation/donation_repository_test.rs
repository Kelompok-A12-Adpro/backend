#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::donation::{Donation, NewDonationRequest};
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use std::env;
    use once_cell::sync::Lazy;


    static TEST_POOL: Lazy<PgPool> = Lazy::new(|| {
        dotenvy::dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for integration tests");
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(&database_url)
            .expect("Failed to create test database pool")
    });

    async fn clear_donations_table(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("TRUNCATE TABLE donations RESTART IDENTITY CASCADE")
            .execute(pool)
            .await?;
        Ok(())
    }

    fn get_repository() -> PgDonationRepository {
        PgDonationRepository::new(TEST_POOL.clone())
    }

    #[tokio::test]
    async fn test_create_donation_success() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let user_id = 1;
        let new_donation_req = NewDonationRequest {
            campaign_id: 101,
            amount: 50.0,
            message: Some("Test donation".to_string()),
        };

        let result = repo.create(user_id, &new_donation_req).await;
        assert!(result.is_ok(), "Failed to create donation: {:?}", result.err());

        let created_donation = result.unwrap();
        assert!(created_donation.id > 0);
        assert_eq!(created_donation.user_id, user_id);
        assert_eq!(created_donation.campaign_id, new_donation_req.campaign_id);
        assert_eq!(created_donation.amount, new_donation_req.amount);
        assert_eq!(created_donation.message, new_donation_req.message);
        assert!(created_donation.created_at < Utc::now() + chrono::Duration::seconds(5));
        assert!(created_donation.created_at > Utc::now() - chrono::Duration::seconds(5));
    }

    #[tokio::test]
    async fn test_create_donation_no_message() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let user_id = 2;
        let new_donation_req = NewDonationRequest {
            campaign_id: 102,
            amount: 25.0,
            message: None,
        };

        let created_donation = repo.create(user_id, &new_donation_req).await.unwrap();
        assert_eq!(created_donation.message, None);
    }

    #[tokio::test]
    async fn test_find_by_id_exists() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let user_id = 3;
        let new_donation_req = NewDonationRequest {
            campaign_id: 103,
            amount: 75.0,
            message: Some("Find me".to_string()),
        };
        let created_donation = repo.create(user_id, &new_donation_req).await.unwrap();

        let found_donation_opt = repo.find_by_id(created_donation.id).await.unwrap();
        assert!(found_donation_opt.is_some());
        let found_donation = found_donation_opt.unwrap();
        assert_eq!(found_donation.id, created_donation.id);
        assert_eq!(found_donation.message, Some("Find me".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_id_not_exists() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let found_donation_opt = repo.find_by_id(99999).await.unwrap();
        assert!(found_donation_opt.is_none());
    }

    #[tokio::test]
    async fn test_find_by_campaign() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let campaign_id_target = 201;
        let campaign_id_other = 202;

        repo.create(1, &NewDonationRequest { campaign_id: campaign_id_target, amount: 10.0, message: None }).await.unwrap();
        repo.create(2, &NewDonationRequest { campaign_id: campaign_id_target, amount: 20.0, message: Some("Msg1".into()) }).await.unwrap();
        repo.create(3, &NewDonationRequest { campaign_id: campaign_id_other, amount: 30.0, message: None }).await.unwrap();

        let donations = repo.find_by_campaign(campaign_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.campaign_id == campaign_id_target));

        let donations_other = repo.find_by_campaign(campaign_id_other).await.unwrap();
        assert_eq!(donations_other.len(), 1);

        let donations_none = repo.find_by_campaign(999).await.unwrap();
        assert!(donations_none.is_empty());
    }

    #[tokio::test]
    async fn test_find_by_user() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let user_id_target = 51;
        let user_id_other = 52;

        repo.create(user_id_target, &NewDonationRequest { campaign_id: 301, amount: 15.0, message: None }).await.unwrap();
        repo.create(user_id_target, &NewDonationRequest { campaign_id: 302, amount: 25.0, message: Some("Msg2".into()) }).await.unwrap();
        repo.create(user_id_other, &NewDonationRequest { campaign_id: 303, amount: 35.0, message: None }).await.unwrap();

        let donations = repo.find_by_user(user_id_target).await.unwrap();
        assert_eq!(donations.len(), 2);
        assert!(donations.iter().all(|d| d.user_id == user_id_target));

        let donations_other = repo.find_by_user(user_id_other).await.unwrap();
        assert_eq!(donations_other.len(), 1);

        let donations_none = repo.find_by_user(999).await.unwrap();
        assert!(donations_none.is_empty());
    }

    #[tokio::test]
    async fn test_update_message_success() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let user_id = 61;
        let initial_donation = repo.create(user_id, &NewDonationRequest {
            campaign_id: 401,
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
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let user_id = 62;
        let initial_donation = repo.create(user_id, &NewDonationRequest {
            campaign_id: 402,
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
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let user_id = 63;
        let non_existent_donation_id = 9999;
        let new_message = Some("This won't be set".to_string());
        let rows_affected = repo.update_message(non_existent_donation_id, user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0);
    }

    #[tokio::test]
    async fn test_update_message_user_mismatch() {
        let repo = get_repository();
        clear_donations_table(&TEST_POOL).await.unwrap();

        let owner_user_id = 64;
        let other_user_id = 65;
        let initial_donation = repo.create(owner_user_id, &NewDonationRequest {
            campaign_id: 404,
            amount: 120.0,
            message: Some("Original message by owner".to_string()),
        }).await.unwrap();

        let new_message = Some("Attempted update by other user".to_string());
        let rows_affected = repo.update_message(initial_donation.id, other_user_id, new_message).await.unwrap();
        assert_eq!(rows_affected, 0, "Update should fail if user_id does not match");

        let donation_after_attempt = repo.find_by_id(initial_donation.id).await.unwrap().unwrap();
        assert_eq!(donation_after_attempt.message, Some("Original message by owner".to_string()));
    }
}