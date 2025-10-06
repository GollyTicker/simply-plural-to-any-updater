use crate::database;
use crate::updater;
use crate::users;
use anyhow::Result;

use rocket::http::Method;
use sp2any_base::meta;
use sqlx::postgres;
use std::env;
use std::time::Duration;

pub fn logging_init() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,sp2any=debug,sp2any_base=debug"),
    )
    .format_timestamp_millis()
    .init();
}

pub async fn application_setup(cli_args: &ApplicationConfig) -> Result<ApplicationSetup> {
    log::info!("# | application_setup");

    let client: reqwest::Client = reqwest::Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(cli_args.request_timeout))
        .build()?;

    log::info!("# | application_setup | client_created");

    let jwt_secret = users::ApplicationJwtSecret {
        inner: cli_args.jwt_application_secret.clone(),
    };

    let application_user_secrets = database::ApplicationUserSecrets {
        inner: cli_args.application_user_secrets.clone(),
    };

    let sp2any_variant_info = meta::SP2AnyVariantInfo {
        variant: cli_args.sp2any_variant.clone(),
        description: cli_args.sp2any_variant_description.clone(),
        show_in_ui: !cli_args.sp2any_variant_hide_in_ui,
    };

    let shared_updaters = updater::UpdaterManager::new(cli_args);

    log::info!("# | application_setup | client_created | basic_info_and_secrets");

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

    log::info!("# | application_setup | client_created | basic_info_and_secrets | cors_configured");

    let db_pool = postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&cli_args.database_url)
        .await?;

    log::info!(
        "# | application_setup | client_created | basic_info_and_secrets | cors_configured | db_connection_created"
    );

    // the macro integrates these files from compile-time!
    let () = sqlx::migrate!("docker/migrations").run(&db_pool).await?;

    log::info!(
        "# | application_setup | client_created | basic_info_and_secrets | cors_configured | db_connection_created | db_migrated"
    );

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

#[derive(Debug, Clone, Default)]
pub struct ApplicationConfig {
    pub database_url: String,
    pub request_timeout: u64,
    pub sp2any_variant: String,
    pub sp2any_variant_description: Option<String>,
    pub sp2any_variant_hide_in_ui: bool,
    pub jwt_application_secret: String,
    pub application_user_secrets: String,
    pub discord_status_message_updater_available: bool,
}

impl ApplicationConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            request_timeout: env::var("REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
            sp2any_variant: env::var("SP2ANY_VARIANT")?,
            sp2any_variant_description: env::var("SP2ANY_VARIANT_DESCRIPTION").ok(),
            sp2any_variant_hide_in_ui: env::var("SP2ANY_VARIANT_HIDE_IN_UI")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
            jwt_application_secret: env::var("JWT_APPLICATION_SECRET")?,
            application_user_secrets: env::var("APPLICATION_USER_SECRETS")?,
            discord_status_message_updater_available: env::var(
                "DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE",
            )
            .unwrap_or_else(|_| "false".to_string())
            .parse()?,
        })
    }
}

#[derive(Clone)]
pub struct ApplicationSetup {
    pub db_pool: sqlx::PgPool,
    pub client: reqwest::Client,
    pub sp2any_variant_info: meta::SP2AnyVariantInfo,
    pub jwt_secret: users::ApplicationJwtSecret,
    pub application_user_secrets: database::ApplicationUserSecrets,
    pub shared_updaters: updater::UpdaterManager,
    pub cors_policy: rocket_cors::Cors,
}
