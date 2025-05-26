use rocket::{Request, request::Outcome};
use rocket::request::FromRequest;
use rocket::http::Status;
use serde::Deserialize;
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Deserialize)]
struct Claims { 
    sub: i32, // User ID
    exp: usize,
    email: String,
    is_admin: bool
}

#[derive(Debug, Clone)]
pub struct AuthUser { 
    pub id: i32,
    pub email: String,
    pub is_admin: bool,
}

#[cfg(not(feature = "test-mode"))]
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
                    id: c.sub,
                    email: c.email,
                    is_admin: c.is_admin,
                })
            }
            Err(_) => Outcome::Forward(Status::Unauthorized),
        }
    }
}

// Test only!!!
#[cfg(feature = "test-mode")]
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(AuthUser {
            id: 1,
            email: "admin@example.com".to_string(),
            is_admin: true,
        })
    }
}