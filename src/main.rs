use anyhow::Result;
use anyhow::anyhow;

use rocket::routes;
use sp2any::meta_api;
use sp2any::metrics;
use sp2any::platforms;
use sp2any::setup;
use sp2any::updater;
use sp2any::users;
use sp2any_base::license;

#[tokio::main]
async fn main() -> Result<()> {
    setup::logging_init();

    println!("{}", license::info_text());

    log::debug!("# | app_setup");

    let cli_args = setup::ApplicationConfig::from_env()?;

    let app_setup = setup::application_setup(&cli_args).await?;

    log::debug!("# | app_setup | configured");

    let () = updater::api::restart_all_user_updaters_for_app_startups(app_setup.clone()).await?;

    log::debug!("# | app_setup | configured | updaters_restarted");

    log::debug!("# | app_setup | configured | updaters_restarted | webserver_starting");

    let () = run_webserver(app_setup).await?;

    log::debug!(
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
        .attach(metrics::PROM_METRICS.clone()) // todo. fix /metrics not available
        .attach(setup.cors_policy)
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
        .mount("/metrics", metrics::PROM_METRICS.clone())
        .launch()
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}
