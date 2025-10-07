use crate::database;
use crate::meta_api::HttpResult;
use crate::meta_api::expose_internal_error;
use crate::updater;
use crate::users::config;
use crate::users::jwt;
use rocket::{State, serde::json::Json};
use sqlx::PgPool;

#[get("/api/config/defaults")]
pub fn get_api_config_defaults()
-> HttpResult<Json<config::UserConfigDbEntries<database::Decrypted>>> {
    Ok(Json(config::UserConfigDbEntries::default()))
}

#[get("/api/user/config")]
pub async fn get_api_user_config(
    db_pool: &State<PgPool>,
    jwt: jwt::Jwt,
    app_user_secrets: &State<database::ApplicationUserSecrets>,
) -> HttpResult<Json<config::UserConfigDbEntries<database::Decrypted, database::ValidConstraints>>>
{
    let user_id = jwt.user_id().map_err(expose_internal_error)?;
    log::info!("# | GET /api/user/config | {user_id}");

    let user_config = database::get_user_secrets(db_pool, &user_id, app_user_secrets)
        .await
        .map_err(expose_internal_error)?;

    log::info!("# | GET /api/user/config | {user_id} | got_config");

    Ok(Json(user_config))
}

#[post("/api/user/config_and_restart", data = "<config>")]
pub async fn post_api_user_config(
    config: Json<config::UserConfigDbEntries<database::Decrypted>>,
    jwt: jwt::Jwt,
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
    shared_updaters: &State<updater::UpdaterManager>,
) -> HttpResult<()> {
    let user_id = jwt.user_id().map_err(expose_internal_error)?;
    log::info!("# | POST /api/user/config_and_restart | {user_id}");

    // check that config satisfies contraints
    let (_, valid_db_config) =
        config::create_config_with_strong_constraints(&user_id, client, &config)
            .map_err(expose_internal_error)?;

    log::info!("# | POST /api/user/config_and_restart | {user_id} | config_valid");

    // todo. this config change should be rolled-back, if the update fails!
    let () = database::set_user_config_secrets(
        db_pool,
        &user_id,
        valid_db_config,
        application_user_secrets,
    )
    .await
    .map_err(expose_internal_error)?;

    log::info!("# | POST /api/user/config_and_restart | {user_id} | config_valid | config_saved");

    let () = updater::api::restart_updater_for_user(
        &user_id,
        db_pool,
        application_user_secrets,
        client,
        shared_updaters,
    )
    .await
    .map_err(expose_internal_error)?;

    log::info!(
        "# | POST /api/user/config_and_restart | {user_id} | config_valid | config_saved | updaters_restarted"
    );

    Ok(())
}
