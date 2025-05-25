use rocket::{Request, request::Outcome};
use rocket::request::FromRequest;
use rocket::http::Status;
use serde::Deserialize;
use jsonwebtoken::{decode, DecodingKey, Validation};
use auth::service::user_service::{decode_jwt, Claims};

pub struct AuthUser { 
    pub id: i32
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match req.headers().get_one("Authorization") {
            Some(header) => {
                if let Some(token) = header.strip_prefix("Bearer ") {
                    token
                } else {
                    return Outcome::Forward(Status::Unauthorized);
                }
            }
            None => return Outcome::Forward(Status::Unauthorized),
        };
        
        match decode_jwt(token) {
            Ok(claims) => {
                Outcome::Success(AuthUser { 
                    id: claims.sub,
                })
            }
            Err(_) => Outcome::Forward(Status::Unauthorized),
        }
    }
}