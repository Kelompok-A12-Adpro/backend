use rocket::{Request, request::Outcome};
use rocket::request::FromRequest;
use rocket::http::Status;
use serde::Deserialize;
use jsonwebtoken::{decode, DecodingKey, Validation};

use auth::repository::user_repository::find_user_by_email;
#[derive(Deserialize)]
struct Claims { 
    sub: String, 
    id: i32, 
    is_admin: bool, 
    name: String, 
    exp: usize }

pub struct AuthUser { 
    pub id: i32,
    pub is_admin: bool,
    pub name: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req.headers()
            .get_one("Authorization")
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or("");
        
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
            &Validation::default()
        ) {
            Ok(token_data) => {
                let c = token_data.claims;
                // Langsung pakai data dari JWT, tanpa query database
                Outcome::Success(AuthUser { 
                    id: c.id, 
                    is_admin: c.is_admin, 
                    name: c.name 
                })
            }
            Err(_) => Outcome::Forward(Status::Unauthorized),
        }
    }
}