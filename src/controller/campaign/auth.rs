use rocket::{Request, request::Outcome};
use rocket::request::FromRequest;
use rocket::http::Status;
use serde::Deserialize;
use jsonwebtoken::{decode, DecodingKey, Validation};

use auth::repository::user_repository::find_user_by_email;

#[derive(Deserialize)]
struct Claims { 
    sub: String, 
    exp: usize 
}

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
            &DecodingKey::from_secret(std::env::var("JWT_SECRET")
                .unwrap_or_default().as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => {
                let sub = data.claims.sub.clone();
                // Jika sub angka → pakai sebagai user_id,
                // kalau bukan angka → lookup by email
                let (id, name, is_admin) = if let Ok(id) = sub.parse::<i32>() {
                    (id, format!("User {}", id), false)
                } else {
                    match find_user_by_email(&sub).await {
                        Some(u) => (u.id, u.name.clone(), u.is_admin),
                        None    => return Outcome::Forward(Status::Unauthorized),
                    }
                };
                Outcome::Success(AuthUser { id, is_admin, name })
            }
            Err(_) => Outcome::Forward(Status::Unauthorized),
        }
    }
}