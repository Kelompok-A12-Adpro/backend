use crate::errors::AppError;
use crate::model::admin::new_campaign_subs::NewCampaignSubscription;
use async_trait::async_trait;

#[async_trait] // Observer repo
pub trait NewCampaignSubscriptionRepository: Send + Sync {
    async fn subscribe(&self, user_email: String) -> Result<(), AppError>;
    async fn unsubscribe(&self, user_email: String) -> Result<(), AppError>;
    async fn get_subscribers(&self) -> Result<Vec<NewCampaignSubscription>, AppError>;
}

pub struct DbNewCampaignSubscriptionRepository {
    pool: sqlx::PgPool,
}

impl DbNewCampaignSubscriptionRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        DbNewCampaignSubscriptionRepository { pool }
    }
}

#[async_trait]
impl NewCampaignSubscriptionRepository for DbNewCampaignSubscriptionRepository {
    async fn subscribe(&self, user_email: String) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO announcement_subscription (user_email) VALUES ($1)",
            user_email
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn unsubscribe(&self, user_email: String) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM announcement_subscription WHERE user_email = $1",
            user_email
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_subscribers(&self) -> Result<Vec<NewCampaignSubscription>, AppError> {
        let rows = sqlx::query!("SELECT user_email, start_at FROM announcement_subscription")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let subscribers: Vec<NewCampaignSubscription> = rows
            .into_iter()
            .map(|row| NewCampaignSubscription {
                user_email: row.user_email,
                start_at: row.start_at.and_utc(),
            })
            .collect();

        Ok(subscribers)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::get_test_pool};
    use serial_test::serial;

    async fn create_test_repo() -> DbNewCampaignSubscriptionRepository {
        let pool = get_test_pool().await;
        reset_test_db(&pool).await.expect("Failed to setup test schema");
        DbNewCampaignSubscriptionRepository { pool }
    }

    async fn reset_test_db(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        let cleanup_result = sqlx::query(
            "TRUNCATE TABLE notification_user, notification, announcement_subscription RESTART IDENTITY CASCADE"
        ).execute(pool).await;
        
        // If TRUNCATE fails (tables don't exist), create them
        if cleanup_result.is_err() {
            let statements = vec![
                "DROP TABLE IF EXISTS notification_user CASCADE",
                "DROP TABLE IF EXISTS announcement_subscription CASCADE", 
                "DROP TABLE IF EXISTS notification CASCADE",
                r#"CREATE TABLE notification (
                    id SERIAL PRIMARY KEY,
                    title VARCHAR(255) NOT NULL,
                    content VARCHAR(255) NOT NULL,
                    created_at timestamp NOT NULL DEFAULT NOW(),
                    target_type VARCHAR(255) NOT NULL
                        DEFAULT 'AllUsers'
                        CHECK (target_type IN (
                            'AllUsers',
                            'SpecificUser',
                            'Fundraisers',
                            'Donors',
                            'NewCampaign'
                        ))
                )"#,
                r#"CREATE TABLE announcement_subscription (
                    user_email VARCHAR(255) NOT NULL PRIMARY KEY,
                    start_at timestamp NOT NULL DEFAULT NOW()
                )"#,
                r#"CREATE TABLE notification_user (
                    user_email VARCHAR(255) NOT NULL,
                    announcement_id INT NOT NULL,
                    created_at timestamp NOT NULL DEFAULT NOW(),
                    PRIMARY KEY (user_email, announcement_id),
                    FOREIGN KEY (announcement_id) REFERENCES notification(id) ON DELETE CASCADE
                )"#,
            ];

            for statement in statements {
                sqlx::query(statement).execute(pool).await?; // Run create schema if TRUNCATE fails
            }
        }

        Ok(())
    }

    async fn cleanup_test_data(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        // Ensure each test cleans up its data
        let _ = sqlx::query(
            "TRUNCATE TABLE notification_user, notification, announcement_subscription RESTART IDENTITY CASCADE"
        )
        .execute(pool)
        .await;

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_new_campaign_subscription_repo() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let email = "greg@gmail.com";
        
        let subscribe_result = repo.subscribe(email.to_string()).await;
        assert!(subscribe_result.is_ok(), "Failed to subscribe: {:?}", subscribe_result);

        let subscribers = repo.get_subscribers().await;
        assert!(subscribers.is_ok(), "Failed to get subscribers: {:?}", subscribers);

        let subscribers = subscribers.unwrap();
        assert_eq!(subscribers.len(), 1, "Expected 1 subscriber");
        assert_eq!(subscribers[0].user_email, email, "Subscriber email mismatch");
        assert!(!subscribers[0].start_at.to_string().is_empty(), "Start time should be set");
    }

    #[tokio::test]
    #[serial]
    async fn test_new_campaign_subscription_unsubscribe() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let email = "greg@gmail.com";

        let subscribe_result = repo.subscribe(email.to_string()).await;
        assert!(subscribe_result.is_ok(), "Failed to subscribe: {:?}", subscribe_result);

        let subscribers = repo.get_subscribers().await;
        assert!(subscribers.is_ok(), "Failed to get subscribers: {:?}", subscribers);

        let subscribers = subscribers.unwrap();
        assert_eq!(subscribers.len(), 1, "Expected 1 subscriber");

        let unsubscribe_result = repo.unsubscribe(email.to_string()).await;
        assert!(unsubscribe_result.is_ok(), "Failed to unsubscribe: {:?}", unsubscribe_result);

        let subscribers = repo.get_subscribers().await;
        assert!(subscribers.is_ok(), "Failed to get subscribers after unsubscribe: {:?}", subscribers);

        let subscribers = subscribers.unwrap();
        assert!(subscribers.is_empty(), "Expected no subscribers after unsubscribe");
    }

    #[tokio::test]
    #[serial]
    async fn test_new_campaign_subscription_get_subscribers_empty() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let subscribers = repo.get_subscribers().await;
        assert!(subscribers.is_ok(), "Failed to get subscribers: {:?}", subscribers);

        let subscribers = subscribers.unwrap();
        assert!(subscribers.is_empty(), "Expected no subscribers initially");
    }
}