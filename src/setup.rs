use crate::database;
use crate::updater;
use crate::users;
use anyhow::Result;
use clap::Parser;
use rocket::http::Method;
use sqlx::postgres;
use std::time::Duration;

pub async fn application_setup(cli_args: &CliArgs) -> Result<ApplicationSetup> {
    let db_pool = postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&cli_args.database_url)
        .await?;

    let client: reqwest::Client = reqwest::Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(cli_args.request_timeout))
        .build()?;

    let jwt_secret = users::ApplicationJwtSecret {
        inner: cli_args.jwt_application_secret.clone(),
    };

    let application_user_secrets = database::ApplicationUserSecrets {
        inner: cli_args.application_user_secrets.clone(),
    };

    let shared_updaters = updater::UpdaterManager::new(cli_args);

    let allowed_origins = rocket_cors::AllowedOrigins::All;
    let allowed_methods = vec![
        Method::Get,
        Method::Post,
        Method::Options,
        Method::Put,
        Method::Head,
    ]
    .into_iter()
    .map(From::from)
    .collect();

    let cors_policy = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods,
        allowed_headers: rocket_cors::AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    Ok(ApplicationSetup {
        db_pool,
        client,
        jwt_secret,
        application_user_secrets,
        shared_updaters,
        cors_policy,
    })
}

#[derive(Parser, Debug, Clone, Default)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, env)]
    pub database_url: String,

    #[arg(short, long, env, default_value_t = 5)]
    pub request_timeout: u64,

    #[arg(short, long, env)]
    pub jwt_application_secret: String,

    #[arg(short, long, env)]
    pub application_user_secrets: String,

    #[arg(short, long, env, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub discord_status_message_updater_available: bool,
}

#[derive(Clone)]
pub struct ApplicationSetup {
    pub db_pool: sqlx::PgPool,
    pub client: reqwest::Client,
    pub jwt_secret: users::ApplicationJwtSecret,
    pub application_user_secrets: database::ApplicationUserSecrets,
    pub shared_updaters: updater::UpdaterManager,
    pub cors_policy: rocket_cors::Cors,
}
