use crate::model;
use crate::updater_state;
use anyhow::Result;
use clap::Parser;
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

    let jwt_secret = model::ApplicationJwtSecret {
        inner: cli_args.jwt_application_secret.clone(),
    };

    let application_user_secrets = model::ApplicationUserSecrets {
        inner: cli_args.application_user_secrets.clone(),
    };

    let shared_updater_state = updater_state::SharedUpdaters::new();

    Ok(ApplicationSetup {
        db_pool,
        client,
        jwt_secret,
        application_user_secrets,
        shared_updaters: shared_updater_state,
    })
}

#[derive(Parser, Debug, Clone, Default)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long)]
    pub database_url: String,

    #[arg(short, long, default_value_t = 5)]
    pub request_timeout: u64,

    #[arg(short, long)]
    pub jwt_application_secret: String,

    #[arg(short, long)]
    pub application_user_secrets: String,
}

pub struct ApplicationSetup {
    pub db_pool: sqlx::PgPool,
    pub client: reqwest::Client,
    pub jwt_secret: model::ApplicationJwtSecret,
    pub application_user_secrets: model::ApplicationUserSecrets,
    pub shared_updaters: updater_state::SharedUpdaters,
}
