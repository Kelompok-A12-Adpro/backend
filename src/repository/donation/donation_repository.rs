use async_trait::async_trait;
use sqlx::PgPool;
use crate::model::donation::donation::Donation;
use crate::model::donation::donation::NewDonationRequest;
use crate::errors::AppError;

#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError>;
    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError>;
    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError>;
    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError>;
}

pub struct PgDonationRepository {
    pool: PgPool,
}

impl PgDonationRepository {
    pub fn new(pool: PgPool) -> Self {
        PgDonationRepository { pool }
    }
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl DonationRepository for PgDonationRepository {
    async fn create(&self, user_id: i32, new_donation: &NewDonationRequest) -> Result<Donation, AppError> {
        unimplemented!()
    }

    async fn find_by_id(&self, donation_id: i32) -> Result<Option<Donation>, AppError> {
        unimplemented!()
    }

    async fn find_by_campaign(&self, campaign_id: i32) -> Result<Vec<Donation>, AppError> {
        unimplemented!()
    }

    async fn find_by_user(&self, user_id: i32) -> Result<Vec<Donation>, AppError> {
        unimplemented!()
    }

    async fn update_message(&self, donation_id: i32, user_id: i32, message: Option<String>) -> Result<u64, AppError> {
        unimplemented!()
    }
}
