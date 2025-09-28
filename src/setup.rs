use crate::database;
use crate::meta_api;
use crate::updater;
use crate::users;
use anyhow::Result;
use clap::Parser;
use rocket::http::Method;
use sqlx::postgres;
use std::time::Duration;

pub async fn application_setup(cli_args: &CliArgs) -> Result<ApplicationSetup> {
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

    let sp2any_variant_info = meta_api::SP2AnyVariantInfo {
        variant: cli_args.sp2any_variant.clone(),
        description: cli_args.sp2any_variant_description.clone(),
        show_in_ui: !cli_args.sp2any_variant_hide_in_ui,
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

    let db_pool = postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&cli_args.database_url)
        .await?;

    // the macro integrates these files from compile-time!
    let () = sqlx::migrate!("docker/migrations").run(&db_pool).await?;

    Ok(ApplicationSetup {
        db_pool,
        client,
        sp2any_variant_info,
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

    #[arg(long, env, default_value_t = 5)]
    pub request_timeout: u64,

    #[arg(long, env)]
    pub sp2any_variant: String, // e.g. variant in <variant>.sp2any.com

    #[arg(long, env)]
    pub sp2any_variant_description: Option<String>,

    #[arg(long, env, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub sp2any_variant_hide_in_ui: bool,

    #[arg(long, env)]
    pub jwt_application_secret: String,

    #[arg(long, env)]
    pub application_user_secrets: String,

    #[arg(long, env, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub discord_status_message_updater_available: bool,
}

#[derive(Clone)]
pub struct ApplicationSetup {
    pub db_pool: sqlx::PgPool,
    pub client: reqwest::Client,
    pub sp2any_variant_info: meta_api::SP2AnyVariantInfo,
    pub jwt_secret: users::ApplicationJwtSecret,
    pub application_user_secrets: database::ApplicationUserSecrets,
    pub shared_updaters: updater::UpdaterManager,
    pub cors_policy: rocket_cors::Cors,
}
