use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use sqlx::{Type, Decode, Encode, Postgres};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CampaignStatus {
    PendingVerification,
    Active,
    Rejected,
    Completed,
}

impl fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CampaignStatus::PendingVerification => "PendingVerification",
            CampaignStatus::Active              => "Active",
            CampaignStatus::Rejected            => "Rejected",
            CampaignStatus::Completed           => "Completed",
        };
        write!(f, "{}", s)
    }
}

impl Type<Postgres> for CampaignStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

impl<'r> Decode<'r, Postgres> for CampaignStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        match s {
            "PendingVerification" => Ok(CampaignStatus::PendingVerification),
            "Active" => Ok(CampaignStatus::Active),
            "Rejected" => Ok(CampaignStatus::Rejected),
            "Completed" => Ok(CampaignStatus::Completed),
            _ => Err(format!("Unknown campaign status: {}", s).into()),
        }
    }
}

impl<'q> Encode<'q, Postgres> for CampaignStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        <&str as Encode<Postgres>>::encode_by_ref(&self.to_string().as_str(), buf)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: String,
    pub target_amount: i64,
    pub collected_amount: i64,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub image_url: Option<String>,
    pub evidence_url: Option<String>,
    pub evidence_uploaded_at: Option<DateTime<Utc>>,
    pub status: CampaignStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}