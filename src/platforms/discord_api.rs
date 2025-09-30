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
    let user_id_c = user_id.clone();
    log::info!("# | GET /api/user/platform/discord/bridge-events | {user_id}");

    let config = database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;
    let (config, _) = users::create_config_with_strong_constraints(&user_id, client, &config)?;

    let mut fronting_channel = shared_updaters.subscribe_fronter_channel(&user_id)?;

    let foreign_status_channel = shared_updaters.get_foreign_status_channel(&user_id)?;

    let notify = move |s: UpdaterStatus| foreign_status_channel.send(Some((Platform::Discord, s)));

    let ws = ws.config(ws::Config {
        write_buffer_size: 0,
        ..Default::default()
    });

    log::info!("# | GET /api/user/platform/discord/bridge-events | {user_id} | setup");

    let stream = {
        ws::Stream! { ws =>
            let mut ws = ws.fuse();

            #[allow(clippy::needless_continue)]
            loop {
                log::info!("# | fronters_chan <-> WS | {user_id} | Waiting...");
                tokio::select! {
                    message = ws.next() => {
                        log::info!("# | fronters_chan <-> WS | {user_id} | WS received {message:?}");
                        match message {
                            Some(close) if is_closed(&close) => {
                                log::info!("# | fronters_chan <-> WS | {user_id} | WS received | ws_stream_is_closed {close:?}");
                                notify(UpdaterStatus::Error("SP2Any-Bridge -> websocket -> SP2Any-Server | No connection to bridge.".to_owned()));
                                break;
                            },
                            Some(Ok(ws::Message::Text(str))) => {
                                let message: BridgeToServerSseMessage = match serde_json::from_str(&str) {
                                    Ok(s) => {
                                        log::info!("# | fronters_chan <-> WS | {user_id} | WS received | deserialised to {s:?}");
                                        s
                                    },
                                    Err(e) => {
                                        log::info!("# | fronters_chan <-> WS | {user_id} | WS received | deserialise_err {e}");
                                        notify(UpdaterStatus::Error(format!("SP2Any-Bridge -> websocket -> SP2Any-Server | Message deserialisation error: {e}")));
                                        break; // end on reading error
                                    }
                                };
                                notify(message.discord_updater_status);
                                continue;
                            },
                            Some(Ok(unknown_message)) => {
                                log::info!("# | fronters_chan <-> WS | {user_id} | WS received | unknown_msg_type {unknown_message:?}");
                                continue; // unknown message ignored
                            }
                            Some(Err(_)) | None => {
                                log::info!("# | fronters_chan <-> WS | {user_id} | WS received | ending_due_to_error {message:?}");
                                notify(UpdaterStatus::Error(format!("SP2Any-Bridge -> websocket -> SP2Any-Server: Server ending due to websocket error '{message:?}'.")));
                                break;
                            }, // client disconnected
                        }
                    },
                    fronters_msg = fronting_channel.recv() => {
                        log::info!("# | fronters_chan <-> WS | {user_id} | fronters received");
                        if let Some(fronters) = fronters_msg {
                            let rich_presence_result = discord::render_fronts_to_discord_rich_presence(fronters, &config);
                            match rich_presence_result {
                                Ok(rich_presence) => {
                                    let message = ServerToBridgeSseMessage {discord_rich_presence: Some(rich_presence)};
                                    let payload = match rocket::serde::json::to_string(&message) {
                                        Ok(p) => p,
                                        Err(e) => {
                                            log::info!("# | fronters_chan <-> WS | {user_id} | fronters received | serialisation_failed {e}");
                                            notify(UpdaterStatus::Error(format!("SP2Any-Server -> websocket -> SP2Any-Bridge: Server couldn't serialise fronters. Error: {e}")));
                                            break;
                                        }
                                    };
                                    log::info!("# | fronters_chan <-> WS | {user_id} | fronters received | sending_via_websocket");
                                    yield ws::Message::Text(payload);
                                }
                                Err(err) => {
                                    log::info!("# | fronters_chan <-> WS | {user_id} | fronters received | rendering_error {err}");
                                    notify(UpdaterStatus::Error(format!("SP2Any-Server -> websocket -> SP2Any-Bridge: Server rendering error: {err}")));
                                    break;
                                }
                            }
                        } else {
                            log::info!("# | fronters_chan <-> WS | {user_id} | fronters_chan_closed?");
                            notify(UpdaterStatus::Error("SP2Any-Server: Couldn't retrieve any fronters. (internal bug?)".into()));
                            break;
                        }
                    },
                }
            }
            log::info!("# | fronters_chan <-> WS | {user_id} | ws_stream_end");
            yield ws::Message::Close(None);
        }
    };

    log::info!(
        "# | GET /api/user/platform/discord/bridge-events | {user_id_c} | setup | returning_stream"
    );

    Ok(stream)
}

const fn is_closed(x: &Result<ws::Message, ws::result::Error>) -> bool {
    matches!(
        x,
        Ok(ws::Message::Close(_))
            | Err(ws::result::Error::AlreadyClosed | ws::result::Error::ConnectionClosed)
    )
}
