use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::{
    config::UserConfigDbEntries,
    model::{self, Email, EncryptedDbSecret, PasswordHashString, SecretType},
};

#[derive(FromRow)]
pub struct User<Secret>
where
    Secret: SecretType,
{
    pub id: model::UserId,
    pub email: model::Email,
    pub password_hash: model::PasswordHashString,
    pub created_at: chrono::NaiveDateTime,

    pub config: UserConfigDbEntries<Secret>,
}

pub async fn create_user(
    db_pool: &PgPool,
    email: Email,
    password_hash: PasswordHashString,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)",
        email.inner,
        password_hash.inner
    )
    .execute(db_pool)
    .await
    .map(|_| ())
    .map_err(|e| anyhow!(e))
}

pub async fn get_user(db_pool: &PgPool, email: Email) -> Result<User<EncryptedDbSecret>> {
    let config: UserConfigDbEntries<EncryptedDbSecret> = sqlx::query_as(
        "SELECT
            wait_seconds,
            system_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_discord,
            enable_vrchat,
            '' as simply_plural_token,
            '' as discord_token,
            '' as vrchat_username,
            '' as vrchat_password,
            '' as vrchat_cookie,
            '' as discord_base_url,
            '' as simply_plural_base_url
            FROM users WHERE email = $1",
    )
    .bind(&email.inner)
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    let UserInfo {
        id,
        email,
        password_hash,
        created_at,
    } = sqlx::query_as!(
        UserInfo,
        "SELECT
            id,
            email,
            password_hash,
            created_at
            FROM users WHERE email = $1",
        email.inner
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    let user = User::<EncryptedDbSecret> {
        id,
        email,
        password_hash,
        created_at,
        config,
    };

    Ok(user)
}

// todo. add variant with decrypted secrets from db

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct WaitSeconds {
    pub inner: Duration,
}
impl From<Duration> for WaitSeconds {
    fn from(value: Duration) -> Self {
        Self { inner: value }
    }
}

impl From<i32> for WaitSeconds {
    fn from(secs: i32) -> Self {
        Duration::from_secs(secs as u64).into()
    }
}

#[derive(FromRow)]
pub struct UserInfo {
    pub id: model::UserId,
    pub email: model::Email,
    pub password_hash: model::PasswordHashString,
    pub created_at: chrono::NaiveDateTime,
}
