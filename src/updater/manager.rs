use crate::plurality::{self};
use crate::updater::{self, work_loop};
use crate::users::UserId;
use crate::{database, users};
use crate::{int_counter_metric, metric, setup};
use anyhow::{Result, anyhow};
use sp2any_base::communication;
use sp2any_base::updater::UpdaterStatus;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use strum::VariantNames;
use tokio::task::JoinHandle;

type SharedMutable<T> = Arc<Mutex<T>>;
type ThreadSafePerUser<T> = SharedMutable<HashMap<UserId, T>>;

type CancleableTasks = Vec<tokio::task::JoinHandle<()>>;
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
int_counter_metric!(UPDATER_MANAGER_SIMPLY_PLURAL_WEBSOCKET_RELEVANT_CHANGE_MESSAGE_COUNT);

#[derive(Clone)]
pub struct UpdaterManager {
    pub tasks: ThreadSafePerUser<CancleableTasks>,
    pub statuses: ThreadSafePerUser<work_loop::UserUpdatersStatuses>,
    pub fronter_channel: ThreadSafePerUser<FronterChannel>,
    pub foreign_managed_status_channel: ThreadSafePerUser<ForeignStatusChannel>,
    pub discord_status_message_available: bool,
    pub updater_start_time: ThreadSafePerUser<chrono::DateTime<chrono::Utc>>,
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
            updater_start_time: Arc::new(Mutex::new(HashMap::new())),
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

    pub fn fronter_channel_get_most_recent_value(
        &self,
        user_id: &UserId,
    ) -> Result<Option<Vec<plurality::Fronter>>> {
        let receiver = self
            .fronter_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .get(user_id)
            .ok_or_else(|| {
                anyhow!("subscribe_fronter_channel: No fronter channel found for {user_id}")
            })?
            .most_recent_value
            .clone();

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
            .get_mut(user_id)
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
        let simply_plural_websocket_listener_task =
            create_simply_plural_websocket_listener_task(&config);

        let owned_self = self.to_owned();
        let application_user_secrets = application_user_secrets.clone();
        let work_loop_task = tokio::spawn(async move {
            work_loop::run_loop(config, owned_self, &db_pool, &application_user_secrets).await;
        });

        locked_task.insert(
            user_id.clone(),
            vec![
                work_loop_task,
                foreign_status_updater_task,
                simply_plural_websocket_listener_task,
            ],
        );

        self.updater_start_time
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .insert(user_id.clone(), chrono::Utc::now());

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

    // when the updaters are started, then the initial value will be fetched from simply plural into here
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

    #[allow(clippy::significant_drop_tightening)]
    fn updater_active_since(&self, user_id: &UserId) -> Result<chrono::DateTime<chrono::Utc>> {
        let locked = self
            .updater_start_time
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?;

        let start_time = locked
            .get(user_id)
            .ok_or_else(|| anyhow!("updater_active_since: No start time found for {user_id}"))?;

        Ok(*start_time)
    }
}

fn create_simply_plural_websocket_listener_task(
    config: &users::UserConfigForUpdater,
) -> JoinHandle<()> {
    let user_id = config.user_id.clone();
    let sp_token = config.simply_plural_token.clone();

    tokio::spawn(async move {
        if sp_token.secret.is_empty() {
            log::info!("SP WS '{user_id}': Not creating websocket, because token is not set.");
            return;
        }
        plurality::auto_reconnecting_websocket_client_to_simply_plural(
            &user_id.to_string(),
            &sp_token.secret,
            async |message| {
                // currently we only want to listen to the websocket events so that we know what kind of messages we're even receiving.
                // they'll be extracted from the logs lateron.
                let changed =
                    plurality::relevantly_changed_based_on_simply_plural_websocket_event(&message)?;
                log::info!("SP WS payload '{user_id}': +{changed} {message}");
                UPDATER_MANAGER_SIMPLY_PLURAL_WEBSOCKET_RELEVANT_CHANGE_MESSAGE_COUNT
                    .with_label_values(&[&user_id.to_string()])
                    .inc();
                Ok(())
            },
        )
        .await;
    })
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

/**
 * Once in a while, we restart the updaters which have been long-living.
 * Sometimes where seems to be some isolated issues where just restarting fixes it all automatically.
 *
 * We only restart the first long living updater, because we want to avoid too many updaters scheduled at the same time
 */
pub async fn restart_first_long_living_updater(
    db_pool: PgPool,
    shared_updaters: UpdaterManager,
    application_user_secrets: database::ApplicationUserSecrets,
) -> Result<()> {
    log::info!("restart_first_long_living_updater");

    let users = database::get_all_users(&db_pool).await?;

    for user_id in users {
        match shared_updaters.updater_active_since(&user_id) {
            Err(e) => log::warn!(
                "restart_first_long_living_updater: Could not get active_since for {user_id}: {e}"
            ),
            Ok(active_since) if is_long_lived(active_since) => {
                log::info!(
                    "restart_first_long_living_updater | restarting {user_id} ({active_since})"
                );
                let config = database::get_user_config_with_secrets(
                    &db_pool,
                    &user_id,
                    &setup::make_client()?,
                    &application_user_secrets,
                )
                .await?;
                let _ = shared_updaters.restart_updater(
                    &user_id,
                    config,
                    db_pool,
                    &application_user_secrets,
                );
                return Ok(());
            }
            Ok(_) => (),
        }
    }

    Ok(())
}

fn is_long_lived(active_since: chrono::DateTime<chrono::Utc>) -> bool {
    let long_lived_duration = std::time::Duration::from_secs(ONE_DAY_AS_SECONDS);

    chrono::Utc::now()
        .signed_duration_since(active_since)
        .to_std()
        .is_ok_and(|duration| duration > long_lived_duration)
}

const ONE_DAY_AS_SECONDS: u64 = 60 * 60 * 24;
