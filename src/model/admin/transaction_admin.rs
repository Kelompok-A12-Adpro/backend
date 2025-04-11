use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Completed,
    Failed,
    Pending,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminTransactionView {
    pub id: i32,
    pub donor_id: Option<i32>,
    pub donor_name: Option<String>,
    pub is_anonymous: bool,
    pub campaign_id: i32,
    pub campaign_name: String,
    pub amount: f64,
    pub transaction_date: DateTime<Utc>,
    pub status: TransactionStatus,
}

#[derive(Debug, Deserialize)]
pub struct TransactionFilterRequest {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub campaign_id: Option<i32>,
    pub status: Option<TransactionStatus>,
}
