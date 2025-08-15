use std::str::FromStr;

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    response, Request, State,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::{
    database::UserInfo,
    model::{ApplicationJwtSecret, PasswordHashString, UserId, UserProvidedPassword},
};

pub fn create_password_hash(password: UserProvidedPassword) -> Result<PasswordHashString> {
    let salt = SaltString::generate(&mut OsRng);

    let pwh = Argon2::default()
        .hash_password(password.inner.as_bytes(), &salt)
        .map_err(|e| anyhow!(e))?;

    Ok(PasswordHashString {
        inner: pwh.to_string(),
    })
}

pub fn verify_password_and_create_token(
    password: UserProvidedPassword,
    user_info: UserInfo,
    jwt_secret: &ApplicationJwtSecret,
) -> Result<JwtString> {
    // don't allow external user to infer what exactly failed

    let pwh = PasswordHash::new(&user_info.password_hash.inner)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    Argon2::default()
        .verify_password(password.inner.as_bytes(), &pwh)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    let token =
        create_token(user_info.id, jwt_secret).map_err(|_| anyhow!("Invalid email/password"))?;

    Ok(token)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    pub claims: Claims,
}

impl Jwt {
    pub fn user_id(&self) -> Result<UserId> {
        self.claims.user_id()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// `SP2Any` `user_id`
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn user_id(&self) -> Result<UserId> {
        let uuid = Uuid::from_str(&self.sub)?;
        Ok(UserId { inner: uuid })
    }
}

#[derive(Serialize)]
pub struct JwtString {
    pub inner: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Jwt {
    type Error = rocket::response::Debug<anyhow::Error>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header_value = req.headers().get_one("authorization");
        if auth_header_value.is_none() {
            return Outcome::Error((
                Status::Unauthorized,
                response::Debug::from(anyhow!("No Jwt provided")),
            ));
        }
        let auth_header_value = auth_header_value.unwrap();

        let jwt_secret = req
            .guard::<&State<ApplicationJwtSecret>>()
            .await
            .map_error(|(err_status, ())| (err_status, response::Debug(anyhow!(err_status))));

        jwt_secret.and_then(|jwt_secret| {
            let token = JwtString {
                inner: auth_header_value
                    .trim_start_matches("Bearer")
                    .trim()
                    .to_owned(),
            };
            match verify_jwt(&token, jwt_secret) {
                Ok(claims) => Outcome::Success(Self { claims }),
                Err(err) => {
                    eprintln!("Token verification failed: {err}");
                    Outcome::Error((
                        Status::Forbidden,
                        response::Debug(anyhow!("Token verification failed")),
                    ))
                }
            }
        })
    }
}

const JWT_VALID_DAYS: i64 = 25;

pub fn create_token(user_id: UserId, jwt_secret: &ApplicationJwtSecret) -> Result<JwtString> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(JWT_VALID_DAYS))
        .expect("invalid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.inner.to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.inner.as_bytes()),
    )?;

    Ok(JwtString { inner: token })
}

pub fn verify_jwt(token: &JwtString, jwt_secret: &ApplicationJwtSecret) -> Result<Claims> {
    let token_data = jsonwebtoken::decode::<Claims>(
        &token.inner,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.inner.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;

    eprintln!("Validated token for user {}", token_data.claims.sub);

    Ok(token_data.claims)
}
