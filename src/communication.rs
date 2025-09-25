use tokio::{sync::broadcast, task::JoinHandle};

pub type HttpResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;

pub fn blocking_abort_and_clear_tasks<T, F>(tasks: &mut Vec<T>, f: F)
where
    F: Fn(T) -> JoinHandle<()>,
{
    let updaters = std::mem::take(tasks);
    for task in updaters {
        let task = f(task);
        let task_id = task.id();
        eprintln!("Aborting... (task_id={task_id})");
        task.abort();
        async_scoped::TokioScope::scope_and_block(|scope| {
            scope.spawn(async {
                let _ = task.await;
            });
        });
        // we can't use await here, because the provided vector from a mutex is not 'static
        // hence the scoped to work with non-'static data
        eprintln!("Aborted. (task_id={task_id})");
    }
}

/// Variation of the `tokio::sync::broadcast` channel, where the sender doesn't
/// care if any receiver is listening. Useful to ensure, that all receivers get only the latest value.
#[derive(Debug, Clone)]
pub struct FireAndForgetChannel<T> {
    inner: broadcast::Sender<T>,
}

#[must_use]
pub fn fire_and_forget_channel<T: Clone>() -> FireAndForgetChannel<T> {
    FireAndForgetChannel {
        inner: broadcast::channel(1).0,
    }
}

impl<T> FireAndForgetChannel<T> {
    /// Sends the value through the channel.
    /// There is no guarantee that any receivers are subscribed and whether they receive the message.
    /// Returns the number of receivers at the moment of the sending. May be 0.
    pub fn send(&self, value: T) -> usize {
        self.inner.send(value).unwrap_or_default()
    }

    #[must_use]
    pub fn subscribe(&self) -> LatestReceiver<T> {
        LatestReceiver {
            inner: self.inner.subscribe(),
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
