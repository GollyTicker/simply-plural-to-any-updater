use std::marker;

use serde::{Deserialize, Serialize};
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{platforms, updater};

pub fn blocking_abort_and_clear_tasks<T, F>(tasks: &mut Vec<T>, f: F)
where
    F: Fn(T) -> JoinHandle<()>,
{
    let updaters = std::mem::take(tasks);
    for task in updaters {
        let task = f(task);
        let task_id = task.id();
        log::debug!("# | task_mgr | task_id={task_id} | aborting...");
        task.abort();
        async_scoped::TokioScope::scope_and_block(|scope| {
            scope.spawn(async {
                let _ = task.await;
            });
        });
        // we can't use await here, because the provided vector from a mutex is not 'static
        // hence the scoped to work with non-'static data
        log::debug!("# | task_mgr | task_id={task_id} | aborting... | aborted_ok");
    }
}

/// Variation of the `tokio::sync::broadcast` channel, where the sender doesn't
/// care if any receiver is listening. Useful to ensure, that all receivers get only the latest value.
#[derive(Debug, Clone)]
pub struct FireAndForgetChannel<T, C = DefaultConfig>
where
    C: Config,
{
    inner: broadcast::Sender<T>,
    pub most_recent_value: Option<T>,
    marker: marker::PhantomData<C>,
}

pub trait Config: Clone {}

#[derive(Clone)]
pub enum DefaultConfig {}

impl Config for DefaultConfig {}

#[derive(Clone)]
pub enum OnlyChanges {}

impl Config for OnlyChanges {}

#[must_use]
pub fn fire_and_forget_channel<T: Clone, C: Config>() -> FireAndForgetChannel<T, C> {
    FireAndForgetChannel {
        inner: broadcast::channel(1).0,
        most_recent_value: None,
        marker: marker::PhantomData,
    }
}

impl<T: Clone, C: Config> FireAndForgetChannel<T, C> {
    #[must_use]
    pub fn subscribe(&self) -> LatestReceiver<T> {
        LatestReceiver {
            inner: self.inner.subscribe(),
        }
    }
}

impl<T: Clone> FireAndForgetChannel<T, DefaultConfig> {
    /// Sends the value through the channel.
    /// There is no guarantee that any receivers are subscribed and whether they receive the message.
    /// Returns the number of receivers at the moment of the sending. May be 0.
    pub fn send(&mut self, new_value: T) -> usize {
        self.most_recent_value = Some(new_value.clone());
        self.inner.send(new_value).unwrap_or_default()
    }
}

impl<T: Clone + Eq> FireAndForgetChannel<T, OnlyChanges> {
    /// Sends the value through the channel.
    /// There is no guarantee that any receivers are subscribed and whether they receive the message.
    /// Returns the number of receivers at the moment of the sending. May be 0.
    ///
    /// In addition, changes are sent only when the new value is different from the old one.
    /// Returns None, if no update was sent.
    pub fn send(&mut self, new_value: T) -> Option<usize> {
        let is_different = self
            .most_recent_value
            .as_ref()
            .map_or_else(|| true, |old| *old != new_value);
        if is_different {
            self.most_recent_value = Some(new_value.clone());
            let receivers = self.inner.send(new_value).unwrap_or_default();
            Some(receivers)
        } else {
            None
        }
    }
}

/// Variation of the `tokio::sync::broadcast` receiver, where we don't care if we miss out
/// on intermediate messages.
#[derive(Debug)]
pub struct LatestReceiver<T> {
    inner: broadcast::Receiver<T>,
}

impl<T: Clone> LatestReceiver<T> {
    /// Await for the next message. Skips outdated messages since the previous await.
    /// Returns None, if the sender is closed and will never return.
    pub async fn recv(&mut self) -> Option<T> {
        loop {
            match self.inner.recv().await {
                Ok(value) => return Some(value),
                Err(broadcast::error::RecvError::Closed) => return None,
                Err(broadcast::error::RecvError::Lagged(_)) => (),
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServerToBridgeSseMessage {
    // If None, then remove old actvity and show nothing.
    pub discord_rich_presence: Option<platforms::DiscordRichPresence>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BridgeToServerSseMessage {
    pub discord_updater_status: updater::UpdaterStatus,
}
