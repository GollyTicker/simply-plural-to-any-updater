use std::sync::{Arc, Mutex};
use tokio::time::sleep;

use crate::{
    config::UserConfig,
    simply_plural::{self},
    updater::{self, Updater, UpdaterState, UpdaterStatus},
};
use anyhow::Result;
use chrono::Utc;

pub fn initial_updaters_state() -> Vec<UpdaterState> {
    updater::implemented_updaters()
        .iter()
        .map(|platform| UpdaterState {
            updater: platform.to_owned(),
            status: UpdaterStatus::Unknown,
        })
        .collect()
}

pub async fn run_loop(config: &UserConfig, updater_state: Arc<Mutex<Vec<UpdaterState>>>) {
    eprintln!("Running Updater ...");

    let mut updaters: Vec<Updater> = updater::implemented_updaters()
        .iter()
        .map(|platform| Updater::new(platform.to_owned()))
        .collect();

    for u in updaters.as_mut_slice() {
        if u.enabled(config) {
            log_error_and_continue(&u.platform().to_string(), u.setup(config).await);
        }
    }

    write_updaters_state_to_arc(config, &updater_state, &updaters);

    loop {
        eprintln!(
            "\n\n======================= UTC {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        log_error_and_continue(
            "Updater Logic",
            loop_logic(config, updaters.as_mut_slice()).await,
        );

        write_updaters_state_to_arc(config, &updater_state, &updaters);

        eprintln!(
            "Waiting {}s for next update trigger...",
            config.wait_seconds.as_secs()
        );

        sleep(config.wait_seconds).await;
    }
}

fn write_updaters_state_to_arc(
    config: &UserConfig,
    updater_state: &Arc<Mutex<Vec<UpdaterState>>>,
    updaters: &[Updater],
) {
    let new_state: Vec<UpdaterState> = updaters.iter().map(|u| u.state(config)).collect();
    match updater_state.try_lock() {
        Err(err) => eprintln!("Error: Failed to lock updater state mutex: {err}"),
        Ok(mut state) => {
            *state = new_state;
        }
    }
}

async fn loop_logic(config: &UserConfig, updaters: &mut [Updater]) -> Result<()> {
    let fronts = simply_plural::fetch_fronts(config).await?;

    for updater in updaters {
        if updater.enabled(config) {
            log_error_and_continue(
                &updater.platform().to_string(),
                updater.update_fronting_status(config, &fronts).await,
            );
        }
    }

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
