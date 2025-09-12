use crate::plurality::{self};
use crate::updater::{self, UpdaterStatus, work_loop};
use crate::users;
use crate::users::UserId;
use crate::{communication, setup};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type SharedMutable<T> = Arc<Mutex<T>>;
type ThreadSafePerUser<T> = SharedMutable<HashMap<UserId, T>>;

type FronterChannel = communication::FireAndForgetChannel<Vec<plurality::Fronter>>;
type ForeignStatusChannel =
    communication::FireAndForgetChannel<Option<(updater::Platform, updater::UpdaterStatus)>>;

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
    pub fn new(cli_args: &setup::CliArgs) -> Self {
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
            .ok_or_else(|| anyhow!("No fronter channel found for {}", user_id))?
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
            .ok_or_else(|| anyhow!("No fronter channel found for  {}", user_id))?
            .send(fronters);

        eprintln!("{user_id}: Send fronter update to {receiver_count} receivers.");

        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn get_foreign_status_channel(
        &self,
        user_id: &UserId,
    ) -> Result<
        communication::FireAndForgetChannel<Option<(updater::Platform, updater::UpdaterStatus)>>,
    > {
        let locked = self
            .foreign_managed_status_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?;

        let specific_channel = locked
            .get(user_id)
            .ok_or_else(|| anyhow!("No foreign status channel found for {}", user_id))?;

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
            .ok_or_else(|| anyhow!("No updaters found!"))?
            .to_owned())
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn notify_updater_statuses(
        &self,
        user_id: &UserId,
        updater_state: work_loop::UserUpdatersStatuses,
    ) -> Result<()> {
        let mut locked = self.statuses.lock().map_err(|e| anyhow!(e.to_string()))?;

        let statuses = locked
            .get_mut(user_id)
            .ok_or_else(|| anyhow!("shouldn't happen. no statuses for user."))?;

        for (p, new_status) in updater_state {
            eprintln!("Setting updater status: {user_id} {p} {new_status}");
            statuses.insert(p, new_status);
        }

        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn restart_updater(
        &self,
        user_id: &UserId,
        config: users::UserConfigForUpdater,
    ) -> Result<()> {
        let mut locked_task = self.tasks.lock().map_err(|e| anyhow!(e.to_string()))?;

        eprintln!("Aborting updaters {user_id}");
        locked_task
            .get(user_id)
            .map(|updaters| updaters.iter().map(tokio::task::JoinHandle::abort));

        let () = self.recreate_fronter_channel(user_id)?;
        let foreign_status_updater_task = self.recreate_foreign_status_channel(user_id)?;
        let () = self.recreate_updater_statuses(user_id)?;

        let owned_self = self.to_owned();
        let work_loop_task = tokio::spawn(async move {
            work_loop::run_loop(config, owned_self).await;
        });

        locked_task.insert(
            user_id.clone(),
            vec![work_loop_task, foreign_status_updater_task],
        );
        eprintln!("Restarted updater {user_id}");

        Ok(())
    }

    fn recreate_fronter_channel(&self, user_id: &UserId) -> Result<()> {
        self.fronter_channel
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .insert(user_id.to_owned(), communication::fire_and_forget_channel()); // old value dropped
        Ok(())
    }

    fn recreate_foreign_status_channel(
        &self,
        user_id: &UserId,
    ) -> Result<tokio::task::JoinHandle<()>> {
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
                        Ok(()) => eprintln!("foreign status update ok."),
                        Err(err) => {
                            eprintln!("ending receiver due to foreign status update err {err}");
                            break;
                        }
                    }
                } else {
                    eprintln!("foreign status updater sender dropped. terminating receiver.");
                    break;
                }
            }
        });

        Ok(foreign_status_updater)
    }

    fn recreate_updater_statuses(&self, user_id: &UserId) -> Result<()> {
        let initially_disabled_status =
            updater::available_updaters(self.discord_status_message_available)
                .into_iter()
                .map(|p| (p, UpdaterStatus::Disabled))
                .collect();
        // todo. we should add status Unknown here because Disabled is only when it's intentionally disabled!

        self.statuses
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .insert(user_id.to_owned(), initially_disabled_status);

        Ok(())
    }
}
