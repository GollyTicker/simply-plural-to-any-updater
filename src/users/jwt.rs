use crate::users::model::UserId;
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    response, Request, State,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ApplicationJwtSecret {
    pub inner: String,
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
        Ok(UserId {
            inner: self.sub.clone().try_into()?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct JwtString {
    pub inner: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Jwt {
    type Error = rocket::response::Debug<anyhow::Error>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn no_jwt_provided_outcome() -> Outcome<Jwt, response::Debug<anyhow::Error>> {
            Outcome::Error((
                Status::Unauthorized,
                response::Debug::from(anyhow!("No Jwt provided")),
            ))
        }

        fn verify_jwt_and_handle_result(
            auth_header_value: &str,
            jwt_secret: &ApplicationJwtSecret,
        ) -> Outcome<Jwt, response::Debug<anyhow::Error>> {
            let token = JwtString {
                inner: auth_header_value
                    .trim_start_matches("Bearer")
                    .trim()
                    .to_owned(),
            };
            match verify_jwt(&token, jwt_secret) {
                Ok(claims) => Outcome::Success(Jwt { claims }),
                Err(err) => {
                    eprintln!("Token verification failed: {err}");
                    Outcome::Error((
                        Status::Forbidden,
                        response::Debug(anyhow!("Token verification failed")),
                    ))
                }
            }
        }

        let jwt_secret = req
            .guard::<&State<ApplicationJwtSecret>>()
            .await
            .map_error(|(err_status, ())| (err_status, response::Debug(anyhow!(err_status))));

        let auth_header_value = req.headers().get_one("authorization");

        auth_header_value.map_or_else(no_jwt_provided_outcome, |auth_header_value| {
            jwt_secret
                .and_then(|jwt_secret| verify_jwt_and_handle_result(auth_header_value, jwt_secret))
        })
    }
}

const JWT_VALID_DAYS: i64 = 25;

pub fn create_token(user_id: &UserId, jwt_secret: &ApplicationJwtSecret) -> Result<JwtString> {
    let expiration: usize = Utc::now()
        .checked_add_signed(Duration::days(JWT_VALID_DAYS))
        .ok_or_else(|| anyhow!("invalid timestamp"))?
        .timestamp()
        .try_into()?;

    let claims = Claims {
        sub: user_id.inner.to_string(),
        exp: expiration,
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
