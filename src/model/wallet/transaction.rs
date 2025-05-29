use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: i32,
    pub wallet_id: i32,
    pub transaction_type: String,  // "topup", "donation", "withdrawal"
    pub amount: f64,
    pub method: Option<String>,     // "GOPAY", "DANA" for topups
    pub phone_number: Option<String>, // Phone number for payment method
    pub campaign_id: Option<i32>,   // Related campaign for donations
    pub created_at: NaiveDateTime,
    pub is_deleted: bool,          // For soft delete
}
