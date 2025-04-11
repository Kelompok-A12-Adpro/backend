use rocket::{get, put, delete, routes};
use rocket::serde::json::Json;
use crate::model::admin::user_admin::{AdminUserView, UserStatus, UserActionRequest};
use crate::errors::AppError;

// Placeholder for simplified controllers
#[get("/admin/users")]
fn get_users() -> Json<Vec<AdminUserView>> {
    Json(Vec::new())
}

#[get("/admin/users/<_user_id>")]
fn get_user_details(_user_id: i32) -> Result<Json<AdminUserView>, AppError> {
    Err(AppError::NotFound("User not found".to_string()))
}

#[put("/admin/users/<_user_id>/block", format = "json", data = "<_action_req>")]
fn block_user(_user_id: i32, _action_req: Json<UserActionRequest>) -> Result<Json<AdminUserView>, AppError> {
    Err(AppError::NotFound("User not found".to_string()))
}

#[put("/admin/users/<_user_id>/unblock", format = "json", data = "<_action_req>")]
fn unblock_user(_user_id: i32, _action_req: Json<UserActionRequest>) -> Result<Json<AdminUserView>, AppError> {
    Err(AppError::NotFound("User not found".to_string()))
}

#[delete("/admin/users/<_user_id>", format = "json", data = "<_action_req>")]
fn delete_user(_user_id: i32, _action_req: Json<UserActionRequest>) -> Result<(), AppError> {
    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_users,
        get_user_details,
        block_user,
        unblock_user,
        delete_user
    ]
}
