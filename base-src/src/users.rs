use std::fmt::Display;

use serde::{Deserialize, Serialize};
use specta;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, sqlx::Type, specta::Type)]
pub struct Email {
    pub inner: String,
}

impl From<String> for Email {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email({})", self.inner)
    }
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
pub struct UserLoginCredentials {
    pub email: Email,
    pub password: UserProvidedPassword,
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
pub struct UserProvidedPassword {
    pub inner: String,
}

#[derive(Serialize, Deserialize, specta::Type)]
pub struct JwtString {
    pub inner: String,
}

impl Display for JwtString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.inner.chars().take(5).collect();
        write!(f, "JwtString({s}...)")
    }
}
