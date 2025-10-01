use crate::plurality::{self};
use crate::updater::{self, work_loop};
use crate::users::UserId;
use crate::{database, users};
use crate::{int_counter_metric, metric, setup};
use anyhow::{Result, anyhow};
use sp2any_base::communication;
use sp2any_base::updater::UpdaterStatus;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use strum::VariantNames;
use tokio::task::JoinHandle;

type SharedMutable<T> = Arc<Mutex<T>>;
type ThreadSafePerUser<T> = SharedMutable<HashMap<UserId, T>>;

type FronterChannel = communication::FireAndForgetChannel<Vec<plurality::Fronter>>;
type ForeignStatusChannel =
    communication::FireAndForgetChannel<Option<(updater::Platform, UpdaterStatus)>>;

int_counter_metric!(UPDATER_MANAGER_RESTART_TOTAL_COUNT);
int_counter_metric!(UPDATER_MANAGER_RESTART_SUCCESS_COUNT);
metric!(
    rocket_prometheus::prometheus::IntGaugeVec,
    UPDATER_PLATFORM_STATUS,
    "updater_platform_status",
    &["user_id", "platform", "status"]
);

#[derive(Clone)]
pub struct UpdaterManager {
    pub tasks: ThreadSafePerUser<work_loop::CancleableUpdater>,
    pub statuses: ThreadSafePerUser<work_loop::UserUpdatersStatuses>,
    pub fronter_channel: ThreadSafePerUser<FronterChannel>,
    pub foreign_managed_status_channel: ThreadSafePerUser<ForeignStatusChannel>,
    pub discord_status_message_available: bool,
}

impl UpdaterManager {
    #[must_use]
    pub fn new(cli_args: &setup::ApplicationConfig) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            statuses: Arc::new(Mutex::new(HashMap::new())),
            fronter_channel: Arc::new(Mutex::new(HashMap::new())),
            discord_status_message_available: cli_args.discord_status_message_updater_available,
            foreign_managed_status_channel: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe_fronter_channel(
        &self,
        user_id: &UserId,
    ) -> Result<communication::LatestReceiver<Vec<plurality::Fronter>>> {
        let receiver = self
            .fronter_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .get(user_id)
            .ok_or_else(|| {
                anyhow!("subscribe_fronter_channel: No fronter channel found for {user_id}")
            })?
            .subscribe();

        Ok(receiver)
    }

    pub fn send_fronter_channel_update(
        &self,
        user_id: &UserId,
        fronters: Vec<plurality::Fronter>,
    ) -> Result<()> {
        let receiver_count = self
            .fronter_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .get(user_id)
            .ok_or_else(|| {
                anyhow!("send_fronter_channel_update: No fronter channel found for  {user_id}")
            })?
            .send(fronters);

        log::info!(
            "# | send_fronter_channel_update | {user_id} | Send fronter update to {receiver_count} receivers."
        );

        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn get_foreign_status_channel(
        &self,
        user_id: &UserId,
    ) -> Result<communication::FireAndForgetChannel<Option<(updater::Platform, UpdaterStatus)>>>
    {
        let locked = self
            .foreign_managed_status_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?;

        let specific_channel = locked.get(user_id).ok_or_else(|| {
            anyhow!("get_foreign_status_channel: No foreign status channel found for {user_id}")
        })?;

        Ok(specific_channel.clone())
    }

    pub fn get_updaters_statuses(
        &self,
        user_id: &UserId,
    ) -> Result<work_loop::UserUpdatersStatuses> {
        Ok(self
            .statuses
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .get(user_id)
            .ok_or_else(|| anyhow!("get_updaters_statuses: No updaters found!"))?
            .to_owned())
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn notify_updater_statuses(
        &self,
        user_id: &UserId,
        updater_state: work_loop::UserUpdatersStatuses,
    ) -> Result<()> {
        let mut locked = self.statuses.lock().map_err(|e| anyhow!(e.to_string()))?;

        let statuses = locked.get_mut(user_id).ok_or_else(|| {
            anyhow!("notify_updater_statuses: shouldn't happen. no statuses for user.")
        })?;

        for (p, new_status) in updater_state {
            log::info!("# | notify_updater_statuses | {user_id} | {p} is {new_status}");

            record_status_in_metrics(user_id, p, &new_status);
            statuses.insert(p, new_status);
        }

        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn restart_updater(
        &self,
        user_id: &UserId,
        config: users::UserConfigForUpdater,
        db_pool: sqlx::PgPool,
        application_user_secrets: &database::ApplicationUserSecrets,
    ) -> Result<()> {
        UPDATER_MANAGER_RESTART_TOTAL_COUNT
            .with_label_values(&[&user_id.to_string()])
            .inc();

        let mut locked_task = self.tasks.lock().map_err(|e| anyhow!(e.to_string()))?;

        log::info!("# | restart_updater | {user_id} | aborting updaters");
        if let Some(task) = locked_task.get_mut(user_id) {
            communication::blocking_abort_and_clear_tasks(task, |x| x);
        }

        let () = self.recreate_fronter_channel(user_id)?;
        let foreign_status_updater_task = self.recreate_foreign_status_channel(user_id)?;
        let () = self.recreate_updater_statuses(user_id, &config)?;

        let owned_self = self.to_owned();
        let application_user_secrets = application_user_secrets.clone();
        let work_loop_task = tokio::spawn(async move {
            work_loop::run_loop(config, owned_self, db_pool, &application_user_secrets).await;
        });

        locked_task.insert(
            user_id.clone(),
            vec![work_loop_task, foreign_status_updater_task],
        );
        log::info!("# | restart_updater | {user_id} | aborting updaters | restarted");
        UPDATER_MANAGER_RESTART_SUCCESS_COUNT
            .with_label_values(&[&user_id.to_string()])
            .inc();

        Ok(())
    }

    fn recreate_fronter_channel(&self, user_id: &UserId) -> Result<()> {
        self.fronter_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .insert(user_id.to_owned(), communication::fire_and_forget_channel()); // old value dropped
        Ok(())
    }

    fn recreate_foreign_status_channel(&self, user_id: &UserId) -> Result<JoinHandle<()>> {
        let new_channel = communication::fire_and_forget_channel();

        self.foreign_managed_status_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .insert(user_id.to_owned(), new_channel.clone()); // old value dropped

        let user_id = user_id.clone();
        let owned_self = self.clone();
        let mut receiver = new_channel.subscribe();
        let foreign_status_updater = tokio::spawn(async move {
            loop {
                if let Some(status) = receiver.recv().await {
                    match owned_self.notify_updater_statuses(&user_id, HashMap::from_iter(status)) {
                        Ok(()) => {
                            log::debug!(
                                "# | foreign_status_updater | {user_id} | status update ok."
                            );
                        }
                        Err(err) => {
                            log::warn!(
                                "# | foreign_status_updater | {user_id} | ending_receiver_due_to_foreign_status_update_err {err}"
                            );
                            break;
                        }
                    }
                } else {
                    log::debug!(
                        "# | foreign_status_updater | {user_id} | foreign_status_updater_sender_droppped terminiating_receiver"
                    );
                    break;
                }
            }
        });

        Ok(foreign_status_updater)
    }

    fn recreate_updater_statuses(
        &self,
        user_id: &UserId,
        config: &users::UserConfigForUpdater,
    ) -> Result<()> {
        let initially_disabled_status =
            updater::available_updaters(self.discord_status_message_available)
                .into_iter()
                .map(|p| (p, updater::initial_status(p, config)))
                .collect();

        self.statuses
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .insert(user_id.to_owned(), initially_disabled_status);

        Ok(())
    }
}

fn record_status_in_metrics(user_id: &UserId, p: updater::Platform, new_status: &UpdaterStatus) {
    // By using strum's EnumVariantNames, we don't need to manually maintain a list of status strings.
    for old_status in UpdaterStatus::VARIANTS {
        UPDATER_PLATFORM_STATUS
            .with_label_values(&[&user_id.to_string(), &p.to_string(), old_status])
            .set(0);
    }

    UPDATER_PLATFORM_STATUS
        .with_label_values(&[&user_id.to_string(), &p.to_string(), new_status.into()])
        .set(1);
}
