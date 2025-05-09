use serde::Serialize;
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, FromRow)]
pub struct Wallet {
    pub id: i32,
    pub user_id: i32,
    pub balance: f64,
    pub updated_at: DateTime<Utc>,
}

