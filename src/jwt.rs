
use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::request::{FromRequest, Outcome};
use serde::{Deserialize, Serialize};
use sqlx::types::uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// `SP2Any` `user_id`
    pub sub: String,
    pub exp: usize,
}

pub fn create_token(user_id: Uuid, secret: &str) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;
    Ok(token)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = anyhow::Error;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        todo!()
    }
}
