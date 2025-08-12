use rocket::request::{FromRequest, Outcome};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};

#[derive(FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct Email {
    pub inner: String,
}

impl From<String> for Email {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct UserId {
    pub inner: Uuid,
}

impl From<Uuid> for UserId {
    fn from(val: Uuid) -> Self {
        Self { inner: val }
    }
}

#[derive(Deserialize, Clone)]
pub struct UserProvidedPassword {
    pub inner: String,
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct PasswordHashString {
    pub inner: String,
}
impl From<String> for PasswordHashString {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

#[derive(Serialize)]
pub struct JwtString {
    pub inner: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtString {
    type Error = anyhow::Error;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        todo!()
    }
}

pub struct ApplicationJwtSecret {
    pub inner: String,
}

#[derive(Deserialize, Clone)]
pub struct UserLoginCredentials {
    pub email: Email,
    pub password: UserProvidedPassword,
}
