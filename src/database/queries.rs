use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::{
    database::constraints,
    database::secrets,
    users,
    users::UserConfigDbEntries,
    users::{Email, UserId},
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

// sqlx can't accept inline types with generics currently
type TMPTYPE1 = UserConfigDbEntries<secrets::Encrypted>;
pub async fn get_user(
    db_pool: &PgPool,
    user_id: &UserId,
) -> Result<UserConfigDbEntries<secrets::Encrypted>> {
    sqlx::query_as!(
        TMPTYPE1,
        "SELECT
            wait_seconds,
            system_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_discord,
            enable_discord_status_message,
            enable_vrchat,
            '' AS simply_plural_token,
            '' AS discord_status_message_token,
            '' AS discord_user_id,
            '' AS discord_oauth_access_token,
            '' AS discord_oauth_refresh_token,
            '' AS vrchat_username,
            '' AS vrchat_password,
            '' AS vrchat_cookie,
            false AS valid_constraints
            FROM users WHERE id = $1",
        user_id.inner
    )
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

    eprintln!("setting secrets: cookie {:?}", config.vrchat_cookie.secret);

    let _ = sqlx::query!(
        "UPDATE users
        SET
            wait_seconds = $2,
            system_name = $3,
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
            enc__discord_user_id = pgp_sym_encrypt($16, $9),
            enc__discord_oauth_access_token = pgp_sym_encrypt($17, $9),
            enc__discord_oauth_refresh_token = pgp_sym_encrypt($18, $9)
        WHERE id = $1",
        user_id.inner,
        config.wait_seconds,
        config.system_name,
        config.status_prefix,
        config.status_no_fronts,
        config.status_truncate_names_to,
        config.enable_discord_status_message,
        config.enable_vrchat,
        secrets_key.inner,
        config.simply_plural_token.secret,
        config.discord_status_message_token.secret,
        config.vrchat_username.secret,
        config.vrchat_password.secret,
        config.vrchat_cookie.secret,
        config.enable_discord,
        config.discord_user_id.secret,
        config.discord_oauth_access_token.secret,
        config.discord_oauth_refresh_token.secret,
    )
    .execute(db_pool)
    .await
    .map_err(|e| anyhow!(e))?;

    Ok(())
}

// sqlx cannot accept inline types with generics currently
type TMPTYPE2 = UserConfigDbEntries<secrets::Decrypted, constraints::ValidConstraints>;
pub async fn get_user_secrets(
    db_pool: &PgPool,
    user_id: &UserId,
    application_user_secret: &secrets::ApplicationUserSecrets,
) -> Result<UserConfigDbEntries<secrets::Decrypted, constraints::ValidConstraints>> {
    let secrets_key = compute_user_secrets_key(user_id, application_user_secret);

    sqlx::query_as!(
        TMPTYPE2,
        "SELECT
            wait_seconds,
            system_name,
            status_prefix,
            status_no_fronts,
            status_truncate_names_to,
            enable_discord,
            enable_discord_status_message,
            enable_vrchat,
            pgp_sym_decrypt(enc__simply_plural_token, $2) AS simply_plural_token,
            pgp_sym_decrypt(enc__discord_status_message_token, $2) AS discord_status_message_token,
            pgp_sym_decrypt(enc__discord_user_id, $2) AS discord_user_id,
            pgp_sym_decrypt(enc__discord_oauth_access_token, $2) AS discord_oauth_access_token,
            pgp_sym_decrypt(enc__discord_oauth_refresh_token, $2) AS discord_oauth_refresh_token,
            pgp_sym_decrypt(enc__vrchat_username, $2) AS vrchat_username,
            pgp_sym_decrypt(enc__vrchat_password, $2) AS vrchat_password,
            pgp_sym_decrypt(enc__vrchat_cookie, $2) AS vrchat_cookie,
            true AS valid_constraints
            FROM users WHERE id = $1",
        user_id.inner,
        secrets_key.inner,
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))
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

pub struct UserInfo {
    pub id: UserId,
    pub email: Email,
    pub password_hash: users::PasswordHashString,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
