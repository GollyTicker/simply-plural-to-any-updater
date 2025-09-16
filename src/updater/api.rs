use crate::communication::HttpResult;
use crate::database;
use crate::setup;
use crate::updater::{manager, work_loop};
use crate::users;
use crate::users::UserId;
use anyhow::Result;
use rocket::{State, serde::json::Json};
use sqlx::PgPool;

#[allow(clippy::needless_pass_by_value)]
#[get("/api/updaters/status")]
pub fn get_api_updaters_status(
    shared_updaters: &State<manager::UpdaterManager>,
    jwt: users::Jwt,
) -> HttpResult<Json<work_loop::UserUpdatersStatuses>> {
    let user_id = jwt.user_id()?;

    let updaters_state: work_loop::UserUpdatersStatuses =
        shared_updaters.get_updaters_statuses(&user_id)?;

    Ok(Json(updaters_state))
}

#[post("/api/updaters/restart")]
pub async fn post_api_updaters_restart(
    jwt: users::Jwt,
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
    shared_updater_state: &State<manager::UpdaterManager>,
) -> HttpResult<()> {
    let user_id = jwt.user_id()?;

    let () = restart_updater_for_user(
        &user_id,
        db_pool,
        application_user_secrets,
        client,
        shared_updater_state,
    )
    .await?;

    Ok(())
}

pub async fn restart_all_user_updaters_for_app_startups(
    setup: setup::ApplicationSetup,
) -> Result<()> {
    eprintln!("Starting all user updaters ...");

    let all_users = database::get_all_users(&setup.db_pool).await?;

    eprintln!("Users: {all_users:?}");

    for user in all_users {
        restart_updater_for_user(
            &user,
            &setup.db_pool,
            &setup.application_user_secrets,
            &setup.client,
            &setup.shared_updaters,
        )
        .await?;
    }

    eprintln!("Starting all user updaters. DONE.");

    Ok(())
}

async fn restart_updater_for_user(
    user_id: &UserId,
    db_pool: &PgPool,
    application_user_secrets: &database::ApplicationUserSecrets,
    client: &reqwest::Client,
    shared_updaters: &manager::UpdaterManager,
) -> Result<()> {
    eprintln!("Restarting user updaters {user_id} ...");

    let db_config = database::get_user_secrets(db_pool, user_id, application_user_secrets).await?;

    let (config, _) = users::create_config_with_strong_constraints(user_id, client, &db_config)?;

    let () = shared_updaters.restart_updater(
        user_id,
        config,
        db_pool.clone(),
        application_user_secrets,
    )?;

    eprintln!("Restarting user updaters {user_id}. DONE.");

    Ok(())
}
