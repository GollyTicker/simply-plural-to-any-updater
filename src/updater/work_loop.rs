use std::collections::HashMap;
use tokio::time::sleep;

use crate::updater::platforms::{Platform, Updater, UpdaterStatus};
use crate::updater::{manager, platforms};
use crate::{database, int_counter_metric, plurality, users};
use anyhow::Result;
use chrono::Utc;

pub type CancleableUpdater = Vec<tokio::task::JoinHandle<()>>;
// NOTE: specta::Type is manually exported in bindings
pub type UserUpdatersStatuses = HashMap<Platform, UpdaterStatus>;
type UserUpdaters = HashMap<Platform, Updater>;

int_counter_metric!(UPDATER_WORK_LOOP_START_TOTAL_COUNT);
int_counter_metric!(UPDATER_WORK_LOOP_START_SUCCESS_COUNT);

pub async fn run_loop(
    config: users::UserConfigForUpdater,
    shared_updaters: manager::UpdaterManager,
    db_pool: sqlx::PgPool,
    application_user_secrets: &database::ApplicationUserSecrets,
) -> ! {
    let user_id = &config.user_id;
    log::info!("# | updater run_loop | {user_id}");

    let mut updaters: UserUpdaters =
        platforms::sp2any_server_updaters(shared_updaters.discord_status_message_available)
            .iter()
            .map(|platform| (platform.to_owned(), Updater::new(platform)))
            .collect();

    for u in updaters.values_mut() {
        if u.enabled(&config) {
            log_error_and_continue(
                &u.platform().to_string(),
                u.setup(&config, &db_pool, application_user_secrets).await,
                &config,
            );
        }
    }

    let statues = get_statuses(&updaters, &config);
    log_error_and_continue(
        "update statues",
        shared_updaters.notify_updater_statuses(user_id, statues),
        &config,
    );

    loop {
        log::info!(
            "\n\n# | updater run_loop | {} | ======================= UTC {}",
            config.user_id,
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        UPDATER_WORK_LOOP_START_TOTAL_COUNT
            .with_label_values(&[&user_id.to_string()])
            .inc();

        log_error_and_continue(
            "Updater Logic",
            loop_logic(&config, &mut updaters, &shared_updaters).await,
            &config,
        );

        let statues = get_statuses(&updaters, &config);
        log_error_and_continue(
            "update statues",
            shared_updaters.notify_updater_statuses(user_id, statues),
            &config,
        );

        log::info!(
            "# | updater run_loop | {} | Waiting {}s for next update trigger...",
            user_id,
            config.wait_seconds.inner.as_secs()
        );
        UPDATER_WORK_LOOP_START_SUCCESS_COUNT
            .with_label_values(&[&user_id.to_string()])
            .inc();

        sleep(config.wait_seconds.inner).await;
    }
}

fn get_statuses(
    updaters: &UserUpdaters,
    config: &users::UserConfigForUpdater,
) -> UserUpdatersStatuses {
    updaters
        .iter()
        .map(|(k, u)| (k.to_owned(), u.status(config)))
        .collect()
}

async fn loop_logic(
    config: &users::UserConfigForUpdater,
    updaters: &mut UserUpdaters,
    shared_updaters: &manager::UpdaterManager,
) -> Result<()> {
    let fronts = plurality::fetch_fronts(config).await?;

    for updater in updaters.values_mut() {
        if updater.enabled(config) {
            log_error_and_continue(
                &updater.platform().to_string(),
                updater.update_fronting_status(config, &fronts).await,
                config,
            );
        }
    }

    shared_updaters.send_fronter_channel_update(&config.user_id, fronts)?;

    Ok(())
}

fn log_error_and_continue(
    loop_part_name: &str,
    res: Result<()>,
    config: &users::UserConfigForUpdater,
) {
    match res {
        Ok(()) => log::info!(
            "# | updater run_loop | {} | {loop_part_name} | ok",
            config.user_id
        ),
        Err(err) => log::info!(
            "# | updater run_loop | {} | {loop_part_name} | skipping due to error {err}",
            config.user_id
        ),
    }
}
