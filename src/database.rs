use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};

use crate::{
    config::UserConfigDbEntries,
    model::{
        self, ApplicationUserSecrets, DecryptedDbSecret, Email, EncryptedDbSecret,
        PasswordHashString, UserId, UserSecretsKey, ValidConstraints,
    },
};

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

pub async fn get_user(
    db_pool: &PgPool,
    user_id: &UserId,
) -> Result<UserConfigDbEntries<EncryptedDbSecret>> {
    sqlx::query_as(
        "SELECT
            wait_seconds,
            system_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_discord,
            enable_vrchat,
            '' AS simply_plural_token,
            '' AS discord_token,
            '' AS vrchat_username,
            '' AS vrchat_password,
            '' AS vrchat_cookie,
            false AS valid_constraints
            FROM users WHERE id = $1",
    )
    .bind(user_id.inner)
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))
}

pub async fn set_user_config_secrets(
    db_pool: &PgPool,
    user_id: UserId,
    config: UserConfigDbEntries<DecryptedDbSecret, ValidConstraints>,
    application_user_secret: &ApplicationUserSecrets,
) -> Result<()> {
    let secrets_key = compute_user_secrets_key(&user_id, application_user_secret);

    let _: Option<UserConfigDbEntries<DecryptedDbSecret>> = sqlx::query_as(
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
            enc__vrchat_cookie = pgp_sym_encrypt($14, $9)
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
    .fetch_optional(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    Ok(())
}

pub async fn get_user_secrets(
    db_pool: &PgPool,
    user_id: &UserId,
    application_user_secret: &ApplicationUserSecrets,
) -> Result<UserConfigDbEntries<DecryptedDbSecret, ValidConstraints>> {
    let secrets_key = compute_user_secrets_key(user_id, application_user_secret);

    sqlx::query_as(
        "SELECT
            wait_seconds,
            system_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_discord,
            enable_vrchat,
            pgp_sym_decrypt(enc__simply_plural_token, $2) AS simply_plural_token,
            pgp_sym_decrypt(enc__discord_token, $2) AS discord_token,
            pgp_sym_decrypt(enc__vrchat_username, $2) AS vrchat_username,
            pgp_sym_decrypt(enc__vrchat_password, $2) AS vrchat_password,
            pgp_sym_decrypt(enc__vrchat_cookie, $2) AS vrchat_cookie,
            true AS valid_constraints
            FROM users WHERE id = $1",
    )
    .bind(user_id.inner)
    .bind(secrets_key.inner)
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))
}

pub async fn get_user_info(db_pool: &PgPool, user_id: UserId) -> Result<UserInfo> {
    sqlx::query_as!(
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
    .map_err(|e| anyhow!(e))
}

fn compute_user_secrets_key(
    user_id: &UserId,
    application_user_secret: &ApplicationUserSecrets,
) -> UserSecretsKey {
    let user_id = user_id.inner.to_string();
    let app_user_secret = &application_user_secret.inner;

    let digest = {
        let mut hasher = Sha256::new();
        hasher.update(user_id);
        hasher.update(app_user_secret);
        hasher.finalize()
    };

    let hex_string = format!("{digest:x}");

    UserSecretsKey { inner: hex_string }
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
    #[allow(clippy::cast_sign_loss)]
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
