use async_trait::async_trait;
use crate::errors::AppError;
use crate::model::admin::user_admin::{AdminUserView, UserStatus};

#[async_trait]
pub trait UserAdminRepository: Send + Sync {
    async fn get_all_users(&self) -> Result<Vec<AdminUserView>, AppError>;
    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<AdminUserView>, AppError>;
    async fn update_user_status(&self, user_id: i32, status: UserStatus, reason: String) -> Result<bool, AppError>;
    async fn delete_user(&self, user_id: i32, reason: String) -> Result<bool, AppError>;
}

// Implementation will be added later
