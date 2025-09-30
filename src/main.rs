#[macro_use]
extern crate rocket;

use anyhow::Result;
use anyhow::anyhow;

use sp2any::license;
use sp2any::{meta_api, platforms, setup, updater, users};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    println!("{}", license::info_text());

    log::info!("# | app_setup");

    let cli_args = setup::ApplicationConfig::from_env()?;

    let app_setup = setup::application_setup(&cli_args).await?;

    log::info!("# | app_setup | configured");

    let () = updater::api::restart_all_user_updaters_for_app_startups(app_setup.clone()).await?;

    log::info!("# | app_setup | configured | updaters_restarted");

    log::info!("# | app_setup | configured | updaters_restarted | webserver_starting");

    let () = run_webserver(app_setup).await?;

    log::info!(
        "# | app_setup | configured | updaters_restarted | webserver_starting | webserver_ended"
    );

    Ok(())
}

async fn run_webserver(setup: setup::ApplicationSetup) -> Result<()> {
    let _ = rocket::build()
        .manage(setup.db_pool)
        .manage(setup.jwt_secret)
        .manage(setup.application_user_secrets)
        .manage(setup.client)
        .manage(setup.shared_updaters)
        .manage(setup.sp2any_variant_info)
        .mount(
            "/",
            routes![
                users::user_api::post_api_user_register,
                users::user_api::post_api_user_login,
                users::user_api::get_api_user_info,
                users::config_api::get_api_user_config,
                users::config_api::post_api_user_config,
                users::config_api::get_api_config_defaults,
                updater::api::get_api_updaters_status,
                platforms::webview_api::get_api_fronting_by_user_id,
                platforms::vrchat_api::post_api_user_platform_vrchat_auth_2fa_request,
                platforms::vrchat_api::post_api_user_platform_vrchat_auth_2fa_resolve,
                platforms::discord_api::get_api_user_platform_discord_bridge_events,
                meta_api::get_api_meta_sp2any_variant,
            ],
        )
        .attach(setup.cors_policy)
        .launch()
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}
