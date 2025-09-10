use std::collections::HashMap;
use tokio::time::sleep;

use crate::updater::platforms::{Platform, Updater, UpdaterStatus};
use crate::updater::{manager, platforms};
use crate::{plurality, users};
use anyhow::Result;
use chrono::Utc;

pub type CancleableUpdater = tokio::task::JoinHandle<()>;
// NOTE: specta::Type is manually exported in bindings
pub type UserUpdatersStatuses = HashMap<Platform, UpdaterStatus>;
type UserUpdaters = HashMap<Platform, Updater>;

pub async fn run_loop(
    config: users::UserConfigForUpdater,
    shared_updaters: manager::UpdaterManager,
) -> ! {
    eprintln!("Running Updater ...");

    let mut updaters: UserUpdaters =
        platforms::available_updaters(shared_updaters.discord_status_message_available)
            .iter()
            .map(|platform| (platform.to_owned(), Updater::new(platform)))
            .collect();

    for u in updaters.values_mut() {
        if u.enabled(&config) {
            log_error_and_continue(&u.platform().to_string(), u.setup(&config).await);
        }
    }

    let statues = get_statuses(&updaters, &config);
    log_error_and_continue(
        "update statues",
        shared_updaters.set_updater_statuses(&config.user_id, statues),
    );

    loop {
        eprintln!(
            "\n\n======================= UTC {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        log_error_and_continue(
            "Updater Logic",
            loop_logic(&config, &mut updaters, &shared_updaters).await,
        );

        let statues = get_statuses(&updaters, &config);
        log_error_and_continue(
            "update statues",
            shared_updaters.set_updater_statuses(&config.user_id, statues),
        );

        eprintln!(
            "Waiting {}s for next update trigger...",
            config.wait_seconds.inner.as_secs()
        );

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
            );
        }
    }

    shared_updaters.send_fronter_channel_update(&config.user_id, fronts)?;

    Ok(())
}

fn log_error_and_continue(loop_part_name: &str, res: Result<()>) {
    match res {
        core::result::Result::Ok(()) => {}
        Err(err) => {
            eprintln!("Error in {loop_part_name}. Skipping. Error: {err}");
        }
    }
}
