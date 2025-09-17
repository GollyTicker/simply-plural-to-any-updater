use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, stream::StreamExt};
use sp2any::for_discord_bridge::FireAndForgetChannel;
use sp2any::platforms::{BridgeToServerSseMessage, ServerToBridgeSseMessage};
use sp2any::updater;
use tokio::net::TcpStream;
use tauri::Manager;
use tauri::async_runtime::JoinHandle;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

use crate::notify_user_on_status;



pub type WsSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type WsReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub fn stream_updater_status_to_ws_messages_task(app: tauri::AppHandle, mut ws_send: WsSender) -> JoinHandle<()> {
    
    tauri::async_runtime::spawn(async move {
        let updater_status_channel = app.state::<FireAndForgetChannel<updater::UpdaterStatus>>();
        let mut updater_status_receiver = updater_status_channel.subscribe();
        log::info!("WS: Starting sender");
        loop {
            let m = updater_status_receiver.recv().await;
            match m {
                Some(status) => {
                    let message = BridgeToServerSseMessage { discord_updater_status: status };
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
                                    "Ending connection to SP2Any. Some problem happened: {err}"
                                ),
                            );
                            break;
                        }
                    }
                },
                None => break,
            }
        }
        log::warn!("update status receiver channel returned None?");
        // end of while okay here. we haven't implemented websocket re-connection yet
    })
}

pub fn stream_ws_messages_to_rich_presence_task(app: tauri::AppHandle, mut ws_read: WsReceiver) -> JoinHandle<()> {
    
    tauri::async_runtime::spawn(async move {
        let rich_presence_channel = app.state::<FireAndForgetChannel<ServerToBridgeSseMessage>>();

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
                                "Connected to SP2Any and receiving updates...",
                            );
                        })
                        .inspect_err(|e| {
                            log::warn!("WS: Error processing SP2Any message: {e}");
                            notify_user_on_status(
                                &app,
                                format!(
                                    "Some problem occurred when applying updates from SP2Any: {e}"
                                ),
                            );
                        });
                    // todo. is it okay to only log this here?
                }
                Ok(x) => log::warn!("Uknown message type: {x:?}"),
                Err(tungstenite::Error::AlreadyClosed) => {
                    log::info!("WS: AlreadyClosed. Ending.");
                    notify_user_on_status(&app, "Connection to SP2Any closed.");
                    break;
                }
                Err(tungstenite::Error::ConnectionClosed) => {
                    log::info!("WS: ConnectionClosed. Ending.");
                    notify_user_on_status(&app, "Connection to SP2Any closed.");
                    break;
                }
                Err(err) => {
                    log::warn!("WS: Ending due to error: {err}");
                    notify_user_on_status(
                        &app,
                        format!("Ending connection to SP2Any due to some problem: {err}"),
                    );
                    break;
                }
            }
        }
        // connection closed. todo. we should try to reconnect in a while.
        notify_user_on_status(
            &app,
            "Connection to SP2Any ended. (We haven't implemented automatic retries yet.)",
        );
    })
}
