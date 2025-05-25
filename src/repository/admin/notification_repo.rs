use crate::errors::AppError;
use crate::model::admin::notification::{
    CreateNotificationRequest, Notification, NotificationTargetType, validate_request,
};
use async_trait::async_trait;
use chrono::Utc;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, AppError>;
    async fn create_notification(&self, notification: &CreateNotificationRequest, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Notification, AppError>;
    async fn push_notification(&self, target: NotificationTargetType, adt_details: Option<String>, notification_id: i32, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, AppError>;
    async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError>;
    async fn get_notification_for_user(&self, user_email: String) -> Result<Vec<Notification>, AppError>;
    async fn get_notification_by_id(&self, notification_id: i32) -> Result<Option<Notification>, AppError>;
    async fn delete_notification(&self, notification_id: i32) -> Result<bool, AppError>;
    async fn delete_notification_user(&self, notification_id: i32, user_email: String) -> Result<bool, AppError>;
}

pub struct DbNotificationRepository {
    pool: sqlx::PgPool,
}

impl DbNotificationRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        DbNotificationRepository { pool }
    }
}

#[async_trait]
impl NotificationRepository for DbNotificationRepository {
    async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, AppError> {
        self.pool.begin().await.map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    async fn create_notification(&self, request: &CreateNotificationRequest, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Notification, AppError> {
        // Validate the request
        validate_request(request).map_err(|e| AppError::ValidationError(e))?;

        // Create a new notification
        let notification = Notification {
            id: 0, // This will be set by the database
            title: request.title.clone(),
            content: request.content.clone(),
            created_at: Utc::now(),
            target_type: request.target_type.clone(),
        };

        let result = sqlx::query!(
            "INSERT INTO notification (title, content, created_at, target_type)
            VALUES ($1, $2, $3, $4) RETURNING id",
            notification.title,
            notification.content,
            notification.created_at.naive_utc(),
            notification.target_type.to_string(),
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Notification {
            id: result.id,
            title: notification.title,
            content: notification.content,
            created_at: notification.created_at,
            target_type: notification.target_type,
        })
    }

    async fn push_notification(&self, target: NotificationTargetType, adt_details: Option<String>, notification_id: i32, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, AppError> {
        match target {
            NotificationTargetType::AllUsers => {
                // All users notification will always be fetched
                Ok(true)
            }
            NotificationTargetType::SpecificUser => {
                let _result = sqlx::query!(
                    "INSERT INTO notification_user (user_email, announcement_id)
                    VALUES ($1, $2)",
                    adt_details.ok_or_else(|| AppError::ValidationError("User email is required for this target".to_string()))?,
                    notification_id
                )
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                Ok(true)
            }
            NotificationTargetType::Fundraisers => {
                // implement later after database ready
                Ok(true)
            }
            NotificationTargetType::Donors => {
                // implement later after database ready
                Ok(true)
            }
            NotificationTargetType::NewCampaign => {
                let _result = sqlx::query!(
                    "INSERT INTO notification_user (user_email, announcement_id)
                    SELECT user_email, $1 FROM announcement_subscription",
                    notification_id
                )
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                Ok(true)
            }
        }
    }

    async fn get_all_notifications(&self) -> Result<Vec<Notification>, AppError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let notifications = sqlx::query!("SELECT * FROM notification")
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(|row| Notification {
                id: row.id,
                title: row.title,
                content: row.content,
                created_at: row.created_at.and_utc(),
                target_type: NotificationTargetType::from_string(&row.target_type)
                    .unwrap_or(NotificationTargetType::AllUsers),
            })
            .collect();

        Ok(notifications)
    }

    async fn get_notification_for_user(&self, user_email: String) -> Result<Vec<Notification>, AppError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let notifications = sqlx::query!(
            "SELECT * FROM notification
                WHERE target_type = $1 OR notification.id IN (
                    SELECT announcement_id
                    FROM notification_user 
                    WHERE user_email = $2)",
            NotificationTargetType::AllUsers.to_string(),
            user_email
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .into_iter()
        .map(|row| Notification {
            id: row.id,
            title: row.title,
            content: row.content,
            created_at: row.created_at.and_utc(),
            target_type: NotificationTargetType::from_string(
                &row.target_type
            ).unwrap_or(NotificationTargetType::AllUsers),
        })
        .collect();

        Ok(notifications)
    }

    async fn get_notification_by_id(
        &self,
        notification_id: i32,
    ) -> Result<Option<Notification>, AppError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let notification =
            sqlx::query!("SELECT * FROM notification WHERE id = $1", notification_id)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?
                .map(|row| Notification {
                    id: row.id,
                    title: row.title,
                    content: row.content,
                    created_at: row.created_at.and_utc(),
                    target_type: NotificationTargetType::from_string(&row.target_type)
                        .unwrap_or(NotificationTargetType::AllUsers),
                });

        Ok(notification)
    }

    async fn delete_notification(&self, notification_id: i32) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let result = sqlx::query!("DELETE FROM notification WHERE id = $1", notification_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_notification_user(&self, notification_id: i32, user_email: String) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let result = sqlx::query!(
            "DELETE FROM notification_user WHERE announcement_id = $1 AND user_email = $2",
            notification_id,
            user_email
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::get_test_pool;
    use serial_test::serial;

    fn get_query_for_target_type<'a>(
        target_type: &'a NotificationTargetType,
        notification_id: i32,
        adt_details: Option<&'a str>,
    ) -> Result<Option<sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>>, AppError>
    {
        match target_type {
            NotificationTargetType::AllUsers => Ok(None),
            NotificationTargetType::Fundraisers => Ok(None),
            NotificationTargetType::Donors => Ok(None),
            NotificationTargetType::NewCampaign => Ok(Some(sqlx::query!(
                "INSERT INTO notification_user (user_email, announcement_id)
                    SELECT user_email, $1 FROM announcement_subscription",
                notification_id
            ))),
            NotificationTargetType::SpecificUser => {
                let user_email = adt_details.ok_or_else(|| {
                    AppError::ValidationError(
                        "User email is required for specific user target".to_string(),
                    )
                })?;
                Ok(Some(sqlx::query!(
                    "INSERT INTO notification_user (user_email, announcement_id)
                    VALUES ($1, $2)",
                    user_email,
                    notification_id
                )))
            }
        }
    }

    // We no longer use the static singleton
    async fn create_test_repo() -> DbNotificationRepository {
        let pool = get_test_pool().await;
        reset_test_db(&pool).await.expect("Failed to setup test schema");
        DbNotificationRepository { pool }
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
    async fn test_create_and_get_notification() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let request = CreateNotificationRequest {
            title: "Test Notification".to_string(),
            content: "This is a test.".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let mut tx = repo.pool.begin().await.expect("Failed to begin transaction");

        let created_notification = repo
            .create_notification(&request, &mut tx)
            .await
            .expect("Failed to create notification");

        let push_result = repo
            .push_notification(
                request.target_type.clone(),
                request.adt_detail.clone(),
                created_notification.id,
                &mut tx,
            )
            .await
            .expect("Failed to push notification");

        assert_eq!(created_notification.title, request.title);
        assert_eq!(created_notification.content, request.content);
        assert_eq!(created_notification.target_type, request.target_type);
        assert!(created_notification.id > 0);

        assert!(push_result, "Notification should be pushed successfully");
        
        tx.commit().await.expect("Failed to commit transaction");

        let fetched_notification = repo
            .get_notification_by_id(created_notification.id)
            .await
            .expect("Failed to get notification");
        assert!(fetched_notification.is_some());
        assert_eq!(fetched_notification.unwrap().id, created_notification.id);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_non_existent_notification() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let fetched_notification = repo
            .get_notification_by_id(999)
            .await
            .expect("Failed to get notification");
        assert!(fetched_notification.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_create_and_get_all_notifications() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let request1 = CreateNotificationRequest {
            title: "Notification 1".to_string(),
            content: "Content 1".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let request2 = CreateNotificationRequest {
            title: "Notification 2".to_string(),
            content: "Content 2".to_string(),
            target_type: NotificationTargetType::Fundraisers,
            adt_detail: Some("1".to_string()),
        };

        let mut tx1 = repo.pool.begin().await.expect("Failed to begin transaction");
        let created1 = repo.create_notification(&request1, &mut tx1).await.unwrap();
        repo.push_notification(
            request1.target_type.clone(),
            request1.adt_detail.clone(),
            created1.id,
            &mut tx1,
        ).await.unwrap();
        tx1.commit().await.expect("Failed to commit transaction");

        let mut tx2 = repo.pool.begin().await.expect("Failed to begin transaction");
        let created2 = repo.create_notification(&request2, &mut tx2).await.unwrap();
        repo.push_notification(
            request2.target_type.clone(),
            request2.adt_detail.clone(),
            created2.id,
            &mut tx2,
        ).await.unwrap();
        tx2.commit().await.expect("Failed to commit transaction");

        let all_notifications = repo
            .get_all_notifications()
            .await
            .expect("Failed to get all notifications");
        
        assert_eq!(all_notifications.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_notification() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let request = CreateNotificationRequest {
            title: "To Be Deleted".to_string(),
            content: "Delete me".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            adt_detail: Some("greg@gmail.com".to_string()),
        };

        let mut tx = repo.pool.begin().await.expect("Failed to begin transaction");

        let created_notification = repo.create_notification(&request, &mut tx).await.unwrap();

        let push_result = repo
            .push_notification(
                request.target_type.clone(),
                request.adt_detail.clone(),
                created_notification.id,
                &mut tx,
            )
            .await
            .expect("Failed to push notification");
        assert!(push_result, "Notification should be pushed successfully");

        tx.commit().await.expect("Failed to commit transaction");

        let notification_id = created_notification.id;

        assert!(
            repo.get_notification_by_id(notification_id)
                .await
                .unwrap()
                .is_some()
        );

        let delete_result = repo
            .delete_notification(notification_id)
            .await
            .expect("Failed to delete notification");
        assert!(delete_result);

        assert!(
            repo.get_notification_by_id(notification_id)
                .await
                .unwrap()
                .is_none()
        );

        let delete_again_result = repo
            .delete_notification(notification_id)
            .await
            .expect("Failed to delete non-existent notification");
        assert!(!delete_again_result);

        // Ensure the notification_user entry is also deleted
        let user_notifications = repo
            .get_notification_for_user("greg@gmail.com".to_string())
            .await
            .expect("Failed to get user notifications");
        assert!(
            user_notifications.is_empty(),
            "User notifications should be empty after deletion"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_create_notification_validation() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let request = CreateNotificationRequest {
            title: "".to_string(), // Empty title to trigger validation error
            content: "Content".to_string(),
            target_type: NotificationTargetType::AllUsers,
            adt_detail: None,
        };

        let mut tx = repo.pool.begin().await.expect("Failed to begin transaction");
        let result = repo.create_notification(&request, &mut tx).await;
        let push_result = repo
            .push_notification(
                request.target_type.clone(),
                request.adt_detail.clone(),
                0, // No notification created due to validation error
                &mut tx,
            )
            .await;

        assert!(result.is_err());
        assert!(push_result.is_err());
        
        match result.err().unwrap() {
            AppError::ValidationError(msg) => {
                assert!(msg == "Title cannot be empty" || msg == "Content cannot be empty")
            }
            _ => panic!("Expected ValidationError"),
        }
        
        tx.rollback().await.expect("Failed to rollback transaction");
    }

    #[tokio::test]
    #[serial]
    async fn test_push_notification() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        // Create subscription for user gregory@gmail.com
        sqlx::query!(
            "INSERT INTO announcement_subscription (user_email) VALUES ($1)",
            "gregory@gmail.com".to_string(),
        )
        .execute(&repo.pool)
        .await
        .expect("Failed to create subscription for user");

        // Create a notification that should be pushed to the user
        let request = CreateNotificationRequest {
            title: "Push Notification".to_string(),
            content: "This notification should be pushed.".to_string(),
            target_type: NotificationTargetType::NewCampaign,
            adt_detail: None,
        };

        let mut tx = repo.pool.begin().await.expect("Failed to begin transaction");

        let created_notification = repo
            .create_notification(&request, &mut tx)
            .await
            .expect("Failed to create notification");

        let push_result = repo
            .push_notification(
                request.target_type.clone(),
                request.adt_detail.clone(),
                created_notification.id,
                &mut tx,
            )
            .await
            .expect("Failed to push notification");
        assert!(push_result, "Notification should be pushed successfully");
        
        tx.commit().await.expect("Failed to commit transaction");

        let notification_id = created_notification.id;

        // Check if the notification was pushed to the user
        let user_notifications = repo
            .get_notification_for_user("gregory@gmail.com".to_string())
            .await
            .expect("Failed to get user notifications");

        assert!(
            !user_notifications.is_empty(),
            "User notifications should not be empty"
        );
        assert!(
            user_notifications.iter().any(|n| n.id == notification_id),
            "User notifications should contain the created notification"
        );

        // Check if the notification is also in the all notification list
        let all_notifications = repo
            .get_all_notifications()
            .await
            .expect("Failed to get all notifications");
        assert!(
            !all_notifications.is_empty(),
            "All notifications should not be empty"
        );
        assert!(
            all_notifications.iter().any(|n| n.id == notification_id),
            "All notifications should contain the created notification"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_notification_user() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let request = CreateNotificationRequest {
            title: "To Be Deleted User".to_string(),
            content: "Delete me for user".to_string(),
            target_type: NotificationTargetType::SpecificUser,
            adt_detail: Some("greg@gmail.com".to_string()),
        };

        let mut tx = repo.pool.begin().await.expect("Failed to begin transaction");

        let created_notification = repo.create_notification(&request, &mut tx).await.unwrap();
        let push_result = repo
            .push_notification(
                request.target_type.clone(),
                request.adt_detail.clone(),
                created_notification.id,
                &mut tx,
            )
            .await
            .expect("Failed to push notification");
        assert!(push_result, "Notification should be pushed successfully");

        tx.commit().await.expect("Failed to commit transaction");
        
        let notification_id = created_notification.id;
        assert!(
            repo.get_notification_by_id(notification_id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            repo.get_notification_for_user("greg@gmail.com".to_string())
                .await
                .unwrap()
                .iter()
                .any(|n| n.id == notification_id)
        );
        
        let delete_result = repo
            .delete_notification_user(notification_id, "greg@gmail.com".to_string())
            .await
            .expect("Failed to delete notification for user");
        assert!(delete_result, "Failed to delete notification for user");
        assert!(
            repo.get_notification_for_user("greg@gmail.com".to_string())
                .await
                .unwrap()
                .iter()
                .all(|n| n.id != notification_id),
            "Notification should not be present for user after deletion"
        );

        // Ensure the notification itself is still present
        assert!(
            repo.get_notification_by_id(notification_id)
                .await
                .unwrap()
                .is_some(),
            "Notification should still exist after user deletion"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_begin_transaction() {
        let repo = create_test_repo().await;
        cleanup_test_data(&repo.pool).await.expect("Failed to cleanup test data");

        let mut tx = repo.begin_transaction().await.expect("Failed to begin transaction");

        let result = sqlx::query!("SELECT 1 as dummy")
            .fetch_one(&mut *tx)
            .await
            .expect("Failed to execute query in transaction");
        assert_eq!(result.dummy, Some(1), "Transaction should execute query successfully");

        // Commit the transaction
        tx.commit().await.expect("Failed to commit transaction");
    }
}
