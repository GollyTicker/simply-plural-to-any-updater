use tracing_subscriber::Layer;

pub(crate) struct TauriLogLayer {
    app_handle: tauri::AppHandle,
    formatter: tracing_subscriber::fmt::format::Full,
}


impl TauriLogLayer {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            app_handle,
            formatter: tracing_subscriber::fmt::format::Full::default(),
        }
    }
}


impl<S> Layer<S> for TauriLogLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        ctx: tracing_subscriber::fmt::FmtContext<'_, S>,
    ) {
        let mut buffer = String::new();
        // Format the event into the buffer
        if self
            .formatter
            .format_event(
                &ctx,
                &mut tracing_subscriber::fmt::writer::WriteAdaptor::new(&mut buffer),
                event,
            )
            .is_ok()
        {
            // Trim newline as the frontend will likely handle lines
            let msg_to_send = buffer.trim_end_matches('\n').to_string();
            if !msg_to_send.is_empty() {
                if let Err(e) = self.app_handle.emit_all("log-message", msg_to_send) {
                    // Fallback if emitting event fails. Avoid `eprintln!` if this layer is part of stderr.
                    // For now, this error is silent in the context of the logger itself.
                    // Consider a more robust way to handle this if critical.
                    let _ = e; // Mark as used
                }
            }
        }
    }
}

