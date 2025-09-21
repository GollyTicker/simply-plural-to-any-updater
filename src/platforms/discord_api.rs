use crate::platforms::{BridgeToServerSseMessage, ServerToBridgeSseMessage, discord};
use crate::updater::{Platform, UpdaterStatus};
use crate::{database, updater, users};
use anyhow::Result;
use rocket::futures::StreamExt;
use rocket::{State, response};
use sqlx::PgPool;

use rocket_ws::{self as ws};

/// This websocket stream sends text messages of the type `ServerToBridgeSseMessage` and
/// receives messages of the type `BridgeToServerSseMessage`.
#[allow(clippy::needless_pass_by_value)]
#[get("/api/user/platform/discord/bridge-events")]
pub async fn get_api_user_platform_discord_bridge_events(
    jwt: users::Jwt,
    ws: ws::WebSocket,
    shared_updaters: &State<updater::UpdaterManager>,
    db_pool: &State<PgPool>,
    client: &State<reqwest::Client>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
) -> Result<ws::Stream!['static], response::Debug<anyhow::Error>> {
    let user_id = jwt.user_id()?;
    let config = database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;
    let (config, _) = users::create_config_with_strong_constraints(&user_id, client, &config)?;

    let mut fronting_channel = shared_updaters.subscribe_fronter_channel(&user_id)?;

    let foreign_status_channel = shared_updaters.get_foreign_status_channel(&user_id)?;

    let notify = move |s: UpdaterStatus| foreign_status_channel.send(Some((Platform::Discord, s)));

    let ws = ws.config(ws::Config {
        write_buffer_size: 0,
        ..Default::default()
    });

    let stream = {
        ws::Stream! { ws =>
            let mut ws = ws.fuse();

            #[allow(clippy::needless_continue)]
            loop {
                eprintln!("{user_id}: Waiting...");
                tokio::select! {
                    message = ws.next() => {
                        eprintln!("{user_id}: WS received: {message:?}");
                        match message {
                            Some(close) if is_closed(&close) => {
                                eprintln!("{user_id}: ended ws stream {close:?}");
                                notify(UpdaterStatus::Error("No connection to bridge.".to_owned()));
                                break;
                            },
                            Some(Ok(ws::Message::Text(str))) => {
                                let message: BridgeToServerSseMessage = match serde_json::from_str(&str) {
                                    Ok(s) => {
                                        eprintln!("{user_id}: WS: message deserialised: {s:?}");
                                        s
                                    },
                                    Err(e) => {
                                        eprintln!("{user_id}: WS message deserialisation error: {e}");
                                        notify(UpdaterStatus::Error(format!("message deserialisation error: {e}")));
                                        break; // end on reading error
                                    }
                                };
                                notify(message.discord_updater_status);
                                continue;
                            },
                            Some(Ok(unknown_message)) => {
                                eprintln!("{user_id}: WS unknown message received: {unknown_message:?}");
                                continue; // unknown message ignored
                            }
                            Some(Err(_)) | None => {
                                eprintln!("{user_id}: Generic WS next() error: {message:?}. Ending.");
                                notify(UpdaterStatus::Error(format!("Generic WS next() error: {message:?}. Ending.")));
                                break;
                            }, // client disconnected
                        }
                    },
                    fronters_msg = fronting_channel.recv() => {
                        eprintln!("{user_id}: fronts received.");
                        if let Some(fronters) = fronters_msg {
                            let rich_presence_result = discord::render_fronts_to_discord_rich_presence(fronters, &config);
                            match rich_presence_result {
                                Ok(rich_presence) => {
                                    let message = ServerToBridgeSseMessage {discord_rich_presence: Some(rich_presence)};
                                    let payload = match rocket::serde::json::to_string(&message) {
                                        Ok(p) => p,
                                        Err(e) => {
                                            eprintln!("{user_id}: Failed to serialize rich presence: {e}");
                                            notify(UpdaterStatus::Error(format!("serialiisation error: {e}")));
                                            break;
                                        }
                                    };
                                    eprintln!("{user_id}: Sending rich presence to bridge via WebSocket...");
                                    yield ws::Message::Text(payload);
                                }
                                Err(err) => {
                                    eprintln!(
                                        "{user_id}: Error while rendering fronts for discord rich presence. Continuing nonetheless. {err}"
                                    );
                                    notify(UpdaterStatus::Error(format!("Rendering error: {err}")));
                                    break;
                                }
                            }
                        } else {
                            eprintln!("{user_id}: Shared updater closed?");
                            notify(UpdaterStatus::Error("fronter channel closed".into()));
                            break;
                        }
                    },
                }
            }
            eprintln!("{user_id}: WebSocket stream end.");
            yield ws::Message::Close(None);
        }
    };

    Ok(stream)
}

const fn is_closed(x: &Result<ws::Message, ws::result::Error>) -> bool {
    matches!(
        x,
        Ok(ws::Message::Close(_))
            | Err(ws::result::Error::AlreadyClosed | ws::result::Error::ConnectionClosed)
    )
}
