use anyhow::Result;
use sqlx::{types::uuid::Uuid, FromRow, PgPool};

#[derive(FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

pub async fn create_user(db_pool: &PgPool, username: &str, password_hash: &str) -> Result<()> {
    // sqlx::query!(
    //     "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
    //     username,
    //     password_hash
    // )
    // .execute(db_pool)
    // .await?;
    Ok(())
}

pub async fn get_user(db_pool: &PgPool, username: &str) -> Result<User> {
    // let user = sqlx::query_as!(
    //     User,
    //     "SELECT id, username, password_hash FROM users WHERE username = $1",
    //     username
    // )
    // .fetch_one(db_pool)
    // .await?;
    // Ok(user)
    todo!()
}
