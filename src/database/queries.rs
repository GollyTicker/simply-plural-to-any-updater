use anyhow::{Result, anyhow};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};

use crate::{
    database::{Decrypted, ValidConstraints, constraints, secrets},
    users::{self, Email, UserConfigDbEntries, UserId},
};

pub async fn create_user(
    db_pool: &PgPool,
    email: Email,
    password_hash: users::PasswordHashString,
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
) -> Result<UserConfigDbEntries<secrets::Encrypted>> {
    sqlx::query_as(
        "SELECT
            wait_seconds,
            website_system_name,
            website_url_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_discord,
            enable_discord_status_message,
            enable_vrchat,
            enable_website,
            '' AS simply_plural_token,
            '' AS discord_status_message_token,
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
    user_id: &UserId,
    config: UserConfigDbEntries<secrets::Decrypted, constraints::ValidConstraints>,
    application_user_secret: &secrets::ApplicationUserSecrets,
) -> Result<()> {
    let secrets_key = compute_user_secrets_key(user_id, application_user_secret);

    let _: Option<UserConfigDbEntries<secrets::Decrypted>> = sqlx::query_as(
        "UPDATE users
        SET
            wait_seconds = $2,
            website_system_name = $3,
            status_prefix = $4,
            status_no_fronts = $5,
            status_truncate_names_to = $6,
            enable_discord_status_message = $7,
            enable_vrchat = $8,
            enc__simply_plural_token = pgp_sym_encrypt($10, $9),
            enc__discord_status_message_token = pgp_sym_encrypt($11, $9),
            enc__vrchat_username = pgp_sym_encrypt($12, $9),
            enc__vrchat_password = pgp_sym_encrypt($13, $9),
            enc__vrchat_cookie = pgp_sym_encrypt($14, $9),
            enable_discord = $15,
            enable_website = $16,
            website_url_name = $17
        WHERE id = $1",
    )
    .bind(user_id.inner)
    .bind(config.wait_seconds)
    .bind(&config.website_system_name)
    .bind(&config.status_prefix)
    .bind(&config.status_no_fronts)
    .bind(config.status_truncate_names_to)
    .bind(config.enable_discord_status_message)
    .bind(config.enable_vrchat)
    .bind(&secrets_key.inner)
    .bind(config.simply_plural_token.map(|s| s.secret))
    .bind(config.discord_status_message_token.map(|s| s.secret))
    .bind(config.vrchat_username.map(|s| s.secret))
    .bind(config.vrchat_password.map(|s| s.secret))
    .bind(config.vrchat_cookie.map(|s| s.secret))
    .bind(config.enable_discord)
    .bind(config.enable_website)
    .bind(config.website_url_name)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    Ok(())
}

pub async fn get_user_secrets(
    db_pool: &PgPool,
    user_id: &UserId,
    application_user_secret: &secrets::ApplicationUserSecrets,
) -> Result<UserConfigDbEntries<secrets::Decrypted, constraints::ValidConstraints>> {
    let secrets_key = compute_user_secrets_key(user_id, application_user_secret);

    sqlx::query_as(
        "SELECT
            wait_seconds,
            website_system_name,
            website_url_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_website,
            enable_discord,
            enable_discord_status_message,
            enable_vrchat,
            pgp_sym_decrypt(enc__simply_plural_token, $2) AS simply_plural_token,
            pgp_sym_decrypt(enc__discord_status_message_token, $2) AS discord_status_message_token,
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

pub async fn modify_user_secrets(
    db_pool: &PgPool,
    user_id: &UserId,
    application_user_secrets: &secrets::ApplicationUserSecrets,
    modify: impl FnOnce(&mut UserConfigDbEntries<Decrypted, ValidConstraints>),
) -> Result<()> {
    let mut user_with_secrets =
        get_user_secrets(db_pool, user_id, application_user_secrets).await?;

    modify(&mut user_with_secrets);

    let unused_client = reqwest::Client::new();

    let (_, new_config) =
        users::create_config_with_strong_constraints(user_id, &unused_client, &user_with_secrets)?;

    let () =
        set_user_config_secrets(db_pool, user_id, new_config, application_user_secrets).await?;

    Ok(())
}

pub async fn get_all_users(db_pool: &PgPool) -> Result<Vec<UserId>> {
    sqlx::query_as!(
        UserId,
        "SELECT
            id AS inner
        FROM users"
    )
    .fetch_all(db_pool)
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

pub async fn find_user_by_website_url_name(
    db_pool: &PgPool,
    website_url_name: &str,
) -> Result<UserInfo> {
    sqlx::query_as!(
        UserInfo,
        "SELECT
            id,
            email,
            password_hash,
            created_at
            FROM users WHERE website_url_name = $1",
        website_url_name
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))
}

fn compute_user_secrets_key(
    user_id: &UserId,
    application_user_secret: &secrets::ApplicationUserSecrets,
) -> secrets::UserSecretsDecryptionKey {
    let user_id = user_id.inner.to_string();
    let app_user_secret = &application_user_secret.inner;

    let digest = {
        let mut hasher = Sha256::new();
        hasher.update(user_id);
        hasher.update(app_user_secret);
        hasher.finalize()
    };

    let hex_string = format!("{digest:x}");

    secrets::UserSecretsDecryptionKey { inner: hex_string }
}

#[derive(FromRow)]
pub struct UserInfo {
    pub id: UserId,
    pub email: Email,
    pub password_hash: users::PasswordHashString,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
