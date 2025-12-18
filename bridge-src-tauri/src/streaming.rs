use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, stream::StreamExt};
use pluralsync_base::for_discord_bridge::{
    BridgeToServerSseMessage, FireAndForgetChannel, ServerToBridgeSseMessage,
};
use pluralsync_base::updater::UpdaterStatus;
use tauri::Manager;
use tauri::async_runtime::JoinHandle;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite};

use crate::{notify_user_on_status, restart_websocket_connection_after_retry_interval};

pub type WsSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type WsReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub fn stream_updater_status_to_ws_messages_task(
    app: tauri::AppHandle,
    mut ws_send: WsSender,
) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let updater_status_channel = app.state::<FireAndForgetChannel<UpdaterStatus>>();
        let mut updater_status_receiver = updater_status_channel.subscribe();
        log::info!("WS: Starting sender");
        loop {
            let m = updater_status_receiver.recv().await;
            match m {
                Some(status) => {
                    let message = BridgeToServerSseMessage {
                        discord_updater_status: status,
                    };
                    let json = match serde_json::to_string(&message) {
                        Ok(x) => x,
                        Err(err) => {
                            log::warn!("Serde serialisation error: {err}");
                            continue;
                        }
                    };
                    log::info!("WS: Sending status: {json}");
                    match ws_send.send(Message::Text(json.into())).await {
                        Ok(()) => log::info!("WS: Sent status."),
                        Err(err) => {
                            log::warn!("WS: Closing. Error sending updater status: {err}");
                            let _ = ws_send.close().await; // we don't care for errors while closing
                            notify_user_on_status(
                                &app,
                                format!(
                                    "⚠️ Ending connection to PluralSync. Some problem happened: {err}"
                                ),
                            );
                            break;
                        }
                    }
                }
                None => break,
            }
        }
        log::warn!("update status receiver channel returned None?");
        // end of while okay here. we haven't implemented websocket re-connection yet
    })
}

pub fn stream_ws_messages_to_rich_presence_task(
    app: tauri::AppHandle,
    mut ws_read: WsReceiver,
) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut rich_presence_channel = app
            .state::<FireAndForgetChannel<ServerToBridgeSseMessage>>()
            .inner()
            .clone();

        log::info!("WS: Starting listener");
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    log::info!("WS: Message: '{text}'");
                    let _ = serde_json::from_str::<ServerToBridgeSseMessage>(&text)
                        .map(|p| rich_presence_channel.send(p))
                        .inspect(|_| {
                            notify_user_on_status(
                                &app,
                                "Connected to PluralSync and receiving updates...",
                            );
                        })
                        .inspect_err(|e| {
                            log::warn!("WS: Error processing PluralSync message: {e}");
                            notify_user_on_status(
                                &app,
                                format!(
                                    "⚠️ Some problem occurred when applying updates from PluralSync: {e}"
                                ),
                            );
                        });
                    // todo. is it okay to only log this here?
                }
                Ok(x) => log::warn!("Uknown message type: {x:?}"),
                Err(tungstenite::Error::AlreadyClosed) => {
                    log::info!("WS: AlreadyClosed. Ending.");
                    notify_user_on_status(&app, "⚠️ Connection to PluralSync closed.");
                    break;
                }
                Err(tungstenite::Error::ConnectionClosed) => {
                    log::info!("WS: ConnectionClosed. Ending.");
                    notify_user_on_status(&app, "⚠️ Connection to PluralSync closed.");
                    break;
                }
                Err(err) => {
                    log::warn!("WS: Ending due to error: {err}");
                    notify_user_on_status(
                        &app,
                        format!("⚠️ Ending connection to PluralSync due to some problem: {err}"),
                    );
                    break;
                }
            }
        }
        notify_user_on_status(
            &app,
            "⚠️ Connection to PluralSync ended. Will try again in a moment...".to_string(),
        );
        restart_websocket_connection_after_retry_interval(&app);
    })
}
