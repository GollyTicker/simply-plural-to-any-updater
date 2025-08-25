use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use serde::{Deserialize, Serialize};

use crate::{database, users::jwt};

#[derive(Deserialize, Clone)]
pub struct UserProvidedPassword {
    pub inner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordHashString {
    pub inner: String,
}
impl From<String> for PasswordHashString {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

pub fn create_password_hash(password: &UserProvidedPassword) -> Result<PasswordHashString> {
    let salt = SaltString::generate(&mut OsRng);

    let pwh = Argon2::default()
        .hash_password(password.inner.as_bytes(), &salt)
        .map_err(|e| anyhow!(e))?;

    Ok(PasswordHashString {
        inner: pwh.to_string(),
    })
}

pub fn verify_password_and_create_token(
    password: &UserProvidedPassword,
    user_info: &database::UserInfo,
    jwt_secret: &jwt::ApplicationJwtSecret,
) -> Result<jwt::JwtString> {
    // don't allow external user to infer what exactly failed

    let pwh = PasswordHash::new(&user_info.password_hash.inner)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    Argon2::default()
        .verify_password(password.inner.as_bytes(), &pwh)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    let token = jwt::create_token(&user_info.id, jwt_secret)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    Ok(token)
}
