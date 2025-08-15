use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use argon2::{
    password_hash::{PasswordHasher, Salt},
    Argon2,
};

use crate::{
    config::UserConfigDbEntries,
    model::{
        self, ApplicationUserSecrets, DecryptedDbSecret, Email, EmailOrUserId, EncryptedDbSecret,
        PasswordHashString, SecretType, UserId, UserSecretsKey,
    },
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

pub async fn get_user_id(db_pool: &PgPool, email: Email) -> Result<UserId> {
    sqlx::query_as!(
        UserId,
        "SELECT
            id AS inner
        FROM users WHERE email = $1",
        email.inner
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))
}

pub async fn get_user(db_pool: &PgPool, user_id: UserId) -> Result<User<EncryptedDbSecret>> {
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
            FROM users WHERE id = 1",
    )
    .bind(user_id.inner)
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    get_user_info(db_pool, user_id, config).await
}

pub async fn set_user_config_secrets(
    db_pool: &PgPool,
    user_id: UserId,
    config: UserConfigDbEntries<DecryptedDbSecret>,
    application_user_secret: &ApplicationUserSecrets,
) -> Result<()> {
    let secrets_key = compute_user_secrets_key(&user_id, application_user_secret)?;

    let new_config: UserConfigDbEntries<DecryptedDbSecret> = sqlx::query_as(
        "UPDATE users
        SET
            wait_seconds = $2,
            system_name = $3,
            status_prefix = $4,
            status_no_fronts = $5,
            status_truncate_names_to = $6,
            enable_discord = $7,
            enable_vrchat = $8,
            enc__simply_plural_token = pgp_sym_encrypt($10, $9),
            enc__discord_token = pgp_sym_encrypt($11, $9),
            enc__vrchat_username = pgp_sym_encrypt($12, $9),
            enc__vrchat_password = pgp_sym_encrypt($13, $9),
            enc__vrchat_cookie = pgp_sym_encrypt($14, $9),
        WHERE id = $1",
    )
    .bind(user_id.inner)
    .bind(config.wait_seconds)
    .bind(&config.system_name)
    .bind(&config.status_prefix)
    .bind(&config.status_no_fronts)
    .bind(config.status_truncate_names_to)
    .bind(config.enable_discord)
    .bind(config.enable_vrchat)
    .bind(&secrets_key.inner)
    .bind(
        config
            .simply_plural_token
            .as_ref()
            .map(|s| s.secret.clone()),
    )
    .bind(config.discord_token.as_ref().map(|s| s.secret.clone()))
    .bind(config.vrchat_username.as_ref().map(|s| s.secret.clone()))
    .bind(config.vrchat_password.as_ref().map(|s| s.secret.clone()))
    .bind(config.vrchat_cookie.as_ref().map(|s| s.secret.clone()))
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    if new_config == config {
        Ok(())
    } else {
        Err(anyhow!("Couldn't correctly set config values"))
    }
}

pub async fn get_user_secrets(
    db_pool: &PgPool,
    user_id: UserId,
    application_user_secret: &ApplicationUserSecrets,
) -> Result<User<DecryptedDbSecret>> {
    let secrets_key = compute_user_secrets_key(&user_id, application_user_secret)?;

    let config: UserConfigDbEntries<DecryptedDbSecret> = sqlx::query_as(
        "SELECT
            wait_seconds,
            system_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_discord,
            enable_vrchat,
            pgp_sym_decrypt(enc__simply_plural_token, $2)::TEXT AS simply_plural_token,
            pgp_sym_decrypt(enc__discord_token, $2)::TEXT AS discord_token,
            pgp_sym_decrypt(enc__vrchat_username, $2)::TEXT AS vrchat_username,
            pgp_sym_decrypt(enc__vrchat_password, $2)::TEXT AS vrchat_password,
            pgp_sym_decrypt(enc__vrchat_cookie, $2)::TEXT AS vrchat_cookie,
            '' as discord_base_url,
            '' as simply_plural_base_url
            FROM users WHERE id = $1",
    )
    .bind(user_id.inner)
    .bind(secrets_key.inner)
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    get_user_info(db_pool, user_id, config).await
}

async fn get_user_info<S: SecretType>(
    db_pool: &PgPool,
    user_id: UserId,
    config: UserConfigDbEntries<S>,
) -> Result<User<S>> {
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
            FROM users WHERE id = $1",
        user_id.inner
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    let user = User::<S> {
        id,
        email,
        password_hash,
        created_at,
        config,
    };

    Ok(user)
}

fn compute_user_secrets_key(
    user_id: &UserId,
    application_user_secret: &ApplicationUserSecrets,
) -> Result<UserSecretsKey> {
    let user_id_string = user_id.inner.to_string();

    let user_id_salt: Salt = user_id_string
        .as_str()
        .try_into()
        .map_err(|_| anyhow!("Computing user secrets key failed (1)"))?;

    let pwh = Argon2::default()
        .hash_password(application_user_secret.inner.as_bytes(), user_id_salt)
        .map_err(|_| anyhow!("Computing user secrets key failed (2)"))?;

    Ok(UserSecretsKey {
        inner: pwh.to_string(),
    })
}

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

impl EmailOrUserId {
    fn where_clause(&self) -> (String, String) {
        match self {
            Self::Email(email) => ("email".into(), email.inner.clone()),
            Self::UserId(user_id) => ("id".into(), user_id.inner.to_string()),
        }
    }
}
