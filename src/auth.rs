use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{
    database::User,
    model::{
        ApplicationJwtSecret, JwtString, PasswordHashString, SecretType, UserId,
        UserProvidedPassword,
    },
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

pub fn verify_password_and_create_token<T: SecretType>(
    password: UserProvidedPassword,
    db_user: User<T>,
    jwt_secret: &ApplicationJwtSecret,
) -> Result<JwtString> {
    // don't allow external user to infer what exactly failed

    let pwh = PasswordHash::new(&db_user.password_hash.inner)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    Argon2::default()
        .verify_password(password.inner.as_bytes(), &pwh)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    let token =
        create_token(db_user.id, jwt_secret).map_err(|_| anyhow!("Invalid email/password"))?;

    Ok(token)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// `SP2Any` `user_id`
    pub sub: String,
    pub exp: usize,
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
