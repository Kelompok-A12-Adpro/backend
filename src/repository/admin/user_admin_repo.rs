use async_trait::async_trait;

#[async_trait]
pub trait UserAdminRepository: Send + Sync {}

// Implementation will be added later
