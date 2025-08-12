use anyhow::{anyhow, Result};
use sqlx::{FromRow, PgPool};

use crate::model::{self, Email, PasswordHashString};

#[derive(FromRow)]
pub struct User {
    pub id: model::UserId,
    pub email: model::Email,
    pub password_hash: model::PasswordHashString,
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
    .map(|x| ())
    .map_err(|e| anyhow!(e))
}

pub async fn get_user(db_pool: &PgPool, email: Email) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT id, email, password_hash FROM users WHERE email = $1",
        email.inner
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!(e))
}
