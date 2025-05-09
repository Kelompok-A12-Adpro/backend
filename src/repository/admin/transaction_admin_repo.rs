use async_trait::async_trait;

#[async_trait]
pub trait TransactionAdminRepository: Send + Sync {}

// Implementation will be added later
