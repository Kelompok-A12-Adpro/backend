use crate::errors::AppError;
use crate::model::admin::statistic::{CampaignStat, DataStatistic, DonationStat, RecentDonation, TransactionData};

pub struct StatisticService {
    pub pool: sqlx::PgPool,
}

impl StatisticService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        StatisticService { pool }
    }

    pub async fn get_data_statistic_count(&self) -> Result<DataStatistic, AppError> {
        let result =
            sqlx::query_as!(
                DataStatistic,
                r#"
                    SELECT
                        COALESCE((SELECT COUNT(*) FROM campaigns WHERE status = 'Active'), 0) AS "active_campaigns_count!: i32",
                        COALESCE((SELECT SUM(amount)::bigint FROM donations), 0) AS "total_donations_amount!: i64",
                        COALESCE((SELECT COUNT(*) FROM donations WHERE created_at >= NOW() - INTERVAL '1 day'), 0) AS "daily_transaction_count!: i32",
                        COALESCE((SELECT COUNT(*) FROM donations WHERE created_at >= NOW() - INTERVAL '7 days'), 0) AS "weekly_transaction_count!: i32"
                "#
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_daily_transaction_statistics(&self) -> Result<Vec<TransactionData>, AppError> {
        let result = sqlx::query_as!(
            TransactionData,
            r#"
                SELECT
                    TO_CHAR(DATE_TRUNC('hour', created_at), 'HH24:MI') AS "name!",
                    COUNT(*)::int AS "transactions!",
                    COALESCE(SUM(amount), 0)::bigint AS "amount!"
                FROM donations
                    WHERE created_at >= NOW() - INTERVAL '24 hour'
                GROUP BY DATE_TRUNC('hour', created_at)
                ORDER BY DATE_TRUNC('hour', created_at)
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_weekly_transaction_statistics(
        &self,
    ) -> Result<Vec<TransactionData>, AppError> {
        let result = sqlx::query_as!(
            TransactionData,
            r#"
            SELECT
                TO_CHAR(DATE_TRUNC('day', created_at), 'FMDay') AS "name!",
                COUNT(*)::int AS "transactions!",
                COALESCE(SUM(amount), 0)::bigint AS "amount!"
            FROM donations
            WHERE created_at >= NOW() - INTERVAL '7 days'
            GROUP BY DATE_TRUNC('day', created_at)
            ORDER BY DATE_TRUNC('day', created_at)
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_recent_transactions(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<RecentDonation>, AppError> {
        let result = sqlx::query_as!(
            RecentDonation,
            r#"
                SELECT
                    d.id AS "id!: i32",
                    d.amount AS "amount!: i64",
                    COALESCE(c.name, 'Unknown Campaign') AS "campaign!",
                    TO_CHAR(d.created_at, 'YYYY-MM-DD') AS "date!"
                FROM donations d
                    LEFT JOIN campaigns c ON d.campaign_id = c.id
                ORDER BY d.created_at DESC
                LIMIT $1
            "#,
            limit.unwrap_or(10000) // Default to 10,000 if no limit is provided
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_all_campaigns_with_progress(&self) -> Result<Vec<CampaignStat>, AppError> {
        let result = sqlx::query_as!(
            CampaignStat,
            r#"
                SELECT
                    c.id AS "id!: i32",
                    c.name AS "name!",
                    c.description AS "description!",
                    c.target_amount AS "target_amount!: i64",
                    c.status AS "status!",
                    c.collected_amount AS "current_amount!: i64",
                    CASE 
                        WHEN c.target_amount > 0 THEN 
                            ROUND((c.collected_amount::NUMERIC / c.target_amount::NUMERIC) * 100.0, 2)::DOUBLE PRECISION
                        ELSE 0.0
                    END AS "progress_percentage!: f64"
                FROM campaigns c
                ORDER BY c.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_all_donations(&self) -> Result<Vec<DonationStat>, AppError> {
        let result = sqlx::query_as!(
            DonationStat,
            r#"
                SELECT
                    d.id AS "id!: i32",
                    d.amount AS "amount!: i64",
                    COALESCE(c.id, 0) AS "campaign_id!: i32",
                    COALESCE(c.name, 'Unknown Campaign') AS "campaign_name!",
                    TO_CHAR(d.created_at, 'YYYY-MM-DD') AS "date!"
                FROM donations d
                    LEFT JOIN campaigns c ON d.campaign_id = c.id
                ORDER BY d.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::get_test_pool;
    use serial_test::serial;

    async fn clear_test_data(pool: &sqlx::PgPool) {
        sqlx::query("TRUNCATE TABLE donations, campaigns RESTART IDENTITY CASCADE")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_new() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool.clone());
        assert_eq!(std::ptr::eq(&service.pool, &pool), false);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_data_statistic_count_empty_db() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool);

        let result = service.get_data_statistic_count().await.unwrap();
        assert_eq!(result.active_campaigns_count, 0);
        assert_eq!(result.total_donations_amount, 0);
        assert_eq!(result.daily_transaction_count, 0);
        assert_eq!(result.weekly_transaction_count, 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_data_statistic_count_with_data() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool.clone());

        // Insert test data with proper schema compliance
        sqlx::query("INSERT INTO campaigns (user_id, name, description, target_amount, start_date, end_date, status) VALUES (1, 'Test Campaign', 'Test Description', 10000, NOW(), NOW() + INTERVAL '30 days', 'Active')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO donations (user_id, campaign_id, amount) VALUES (1, 1, 1000)")
            .execute(&pool)
            .await
            .unwrap();

        let result = service.get_data_statistic_count().await.unwrap();
        assert_eq!(result.active_campaigns_count, 1);
        assert_eq!(result.total_donations_amount, 1000);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_daily_transaction_statistics_empty() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool);

        let result = service.get_daily_transaction_statistics().await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_daily_transaction_statistics_with_data() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool.clone());

        // Insert test data with recent timestamp
        sqlx::query("INSERT INTO campaigns (user_id, name, description, target_amount, start_date, end_date, status) VALUES (1, 'Test Campaign', 'Test Description', 10000, NOW(), NOW() + INTERVAL '30 days', 'Active')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO donations (user_id, campaign_id, amount, created_at) VALUES (1, 1, 500, NOW())")
            .execute(&pool)
            .await
            .unwrap();

        let result = service.get_daily_transaction_statistics().await.unwrap();
        assert!(result.len() > 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_weekly_transaction_statistics_empty() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool);

        let result = service.get_weekly_transaction_statistics().await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_weekly_transaction_statistics_with_data() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool.clone());

        // Insert test data
        sqlx::query("INSERT INTO campaigns (user_id, name, description, target_amount, start_date, end_date, status) VALUES (1, 'Test Campaign', 'Test Description', 10000, NOW(), NOW() + INTERVAL '30 days', 'Active')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO donations (user_id, campaign_id, amount, created_at) VALUES (1, 1, 750, NOW())")
            .execute(&pool)
            .await
            .unwrap();

        let result = service.get_weekly_transaction_statistics().await.unwrap();
        assert!(result.len() > 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_recent_transactions_with_default_limit() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool.clone());

        // Insert test data
        sqlx::query("INSERT INTO campaigns (user_id, name, description, target_amount, start_date, end_date, status) VALUES (1, 'Test Campaign', 'Test Description', 10000, NOW(), NOW() + INTERVAL '30 days', 'Active')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO donations (user_id, campaign_id, amount) VALUES (1, 1, 1000)")
            .execute(&pool)
            .await
            .unwrap();

        let result = service.get_recent_transactions(None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].amount, 1000);
        assert_eq!(result[0].campaign, "Test Campaign".to_string());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_recent_transactions_with_custom_limit() {
        let pool = get_test_pool().await;
        clear_test_data(&pool).await;
        let service = StatisticService::new(pool.clone());

        // Insert test data
        sqlx::query("INSERT INTO campaigns (user_id, name, description, target_amount, start_date, end_date, status) VALUES (1, 'Test Campaign', 'Test Description', 10000, NOW(), NOW() + INTERVAL '30 days', 'Active')")
            .execute(&pool)
            .await
            .unwrap();

        for i in 1..=5 {
            sqlx::query("INSERT INTO donations (user_id, campaign_id, amount) VALUES (1, 1, $1)")
                .bind(i * 100)
                .execute(&pool)
                .await
                .unwrap();
        }

        let result = service.get_recent_transactions(Some(3)).await.unwrap();
        assert_eq!(result.len(), 3);
    }
}
