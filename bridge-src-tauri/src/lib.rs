mod discord_bridge;
mod never;

use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use futures::{SinkExt, stream::StreamExt};
use sp2any::for_discord_bridge;
use sp2any::for_discord_bridge::DiscordRichPresence;
use sp2any::for_discord_bridge::FireAndForgetChannel;
use sp2any::license;
use sp2any::updater;
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::Emitter;
use tauri::Manager;
use tauri::async_runtime::{JoinHandle, Mutex};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

// todo. add auto-update capabilities.
// todo. add auto-start capabilities: https://crates.io/crates/auto-launch
// todo. note, that only a single user account is supported for now.

const DEFAULT_SP2ANY_BASE_URL: &str = "https://sp2any.io";
const MEGABYTES: u128 = 10 ^ 6;

fn get_data_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("io", "sp2any", "sp2any.bridge")
        .ok_or_else(|| anyhow!("Failed to get project directories"))
        .map(|p| p.data_local_dir().to_path_buf());

    let data_dir = env::var("SP2ANY_DATA_DIR")
        .map(PathBuf::from)
        .or(proj_dirs)?;

    log::debug!("Data dir: {}", data_dir.to_string_lossy());

    Ok(data_dir)
}

fn get_credentials_path() -> Result<PathBuf> {
    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join("credentials.json"))
}

#[tauri::command]
async fn initiate_discord_rpc_loop(app: tauri::AppHandle) -> () {
    log::debug!("initiate_discord_rpc_loop");
    let rich_presence_channel = app
        .state::<FireAndForgetChannel<DiscordRichPresence>>()
        .inner()
        .clone();
    let updater_status_channel = app
        .state::<FireAndForgetChannel<updater::UpdaterStatus>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn(async move {
        discord_bridge::discord_ipc_loop(&app, rich_presence_channel, updater_status_channel).await;
    });
}

#[tauri::command]
async fn subscribe_to_bridge_channel(
    app: tauri::AppHandle,
    jwt: for_discord_bridge::JwtString,
) -> Result<(), String> {
    log::debug!("subscribe_to_bridge_channel");
    subscribe_to_bridge_channel_anyhow(app, jwt)
        .await
        .map_err(|e| e.to_string())
}

async fn subscribe_to_bridge_channel_anyhow(
    app: tauri::AppHandle,
    jwt: for_discord_bridge::JwtString,
) -> Result<()> {
    let base_url =
        env::var("SP2ANY_BASE_URL").unwrap_or_else(|_| DEFAULT_SP2ANY_BASE_URL.to_owned());
    let ws_url = format!(
        "{}/api/user/platform/discord/bridge-events",
        base_url.replace("http", "ws")
    );

    let mut request = ws_url.clone().into_client_request()?;
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {}", jwt.inner).parse()?);

    log::info!("Connecting to WebSocket at {ws_url}");
    notify_user_on_status(&app, "Connecting to SP2Any to received updates ...");

    // todo. add retries of connections when they are closed etc.
    // This websocket stream receives text messages of the type DiscordRichPresence and
    // sends messages of the type UpdaterStatus.
    let (ws_stream, _) = connect_async(request).await?;
    let (mut ws_send, mut ws_read) = ws_stream.split();

    let app2 = app.clone();
    let forwarder_task = tauri::async_runtime::spawn(async move {
        let updater_status_channel = app2.state::<FireAndForgetChannel<updater::UpdaterStatus>>();
        let mut updater_status_receiver = updater_status_channel.subscribe();
        log::info!("WS: Starting sender");
        loop {
            let m = updater_status_receiver.recv().await;
            match m {
                Some(status) => {
                    let json = match serde_json::to_string(&status) {
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
                                &app2,
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
    });
    register_background_task(app.clone(), forwarder_task).await;

    let app2 = app.clone();
    let receiver_task = tauri::async_runtime::spawn(async move {
        let rich_presence_channel = app2.state::<FireAndForgetChannel<DiscordRichPresence>>();

        log::info!("WS: Starting listener");
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    log::info!("WS: Message: '{text}'");
                    let _ = serde_json::from_str(&text)
                        .map(|p| rich_presence_channel.send(p))
                        .inspect(|_| {
                            notify_user_on_status(
                                &app2,
                                "Connected to SP2Any and receiving updates...",
                            );
                        })
                        .inspect_err(|e| {
                            log::warn!("WS: Error processing SP2Any message: {e}");
                            notify_user_on_status(
                                &app2,
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
                    notify_user_on_status(&app2, "Connection to SP2Any closed.");
                    break;
                }
                Err(tungstenite::Error::ConnectionClosed) => {
                    log::info!("WS: ConnectionClosed. Ending.");
                    notify_user_on_status(&app2, "Connection to SP2Any closed.");
                    break;
                }
                Err(err) => {
                    log::warn!("WS: Ending due to error: {err}");
                    notify_user_on_status(
                        &app2,
                        format!("Ending connection to SP2Any due to some problem: {err}"),
                    );
                    break;
                }
            }
        }
        // connection closed. todo. we should try to reconnect in a while.
        notify_user_on_status(
            &app2,
            "Connection to SP2Any ended. (We haven't implemented automatic retries yet.)",
        );
    });
    register_background_task(app, receiver_task).await;

    Ok(())
}

async fn register_background_task(app: tauri::AppHandle, handle: JoinHandle<()>) {
    let state = app.state::<Mutex<Vec<JoinHandle<()>>>>();
    state.lock().await.push(handle);
}

pub fn notify_user_on_status<S: Into<String>>(app: &tauri::AppHandle, value: S) {
    let _ = app.emit("notify_user_on_status", value.into());
    // we don't care about the success.
}

fn new_background_tasks_container() -> Mutex<Vec<JoinHandle<()>>> {
    Mutex::new(vec![])
}

async fn abort_background_task(app: tauri::AppHandle) -> () {
    log::debug!("abort_background_task");
    let state = app.state::<Mutex<Vec<JoinHandle<()>>>>();
    let mut locked_tasks = state.lock().await;
    locked_tasks
        .iter()
        .for_each(tauri::async_runtime::JoinHandle::abort);
    *locked_tasks = vec![];
}

#[tauri::command]
async fn login(
    creds: for_discord_bridge::UserLoginCredentials,
) -> Result<for_discord_bridge::JwtString, String> {
    log::debug!("login");
    login_anyhow(creds).await.map_err(|e| e.to_string())
}

async fn login_anyhow(
    creds: for_discord_bridge::UserLoginCredentials,
) -> Result<for_discord_bridge::JwtString> {
    let client = reqwest::Client::new();
    let base_url =
        env::var("SP2ANY_BASE_URL").unwrap_or_else(|_| DEFAULT_SP2ANY_BASE_URL.to_owned());
    let login_url = format!("{}{}", base_url, "/api/user/login");

    log::info!("Attempting login: {login_url} with {:?}", &creds.email);

    let jwt_string = client
        .post(login_url)
        .json(&creds)
        .send()
        .await?
        .error_for_status()?
        .json::<for_discord_bridge::JwtString>()
        .await?;

    log::info!("Login successful for {:?}", &creds.email);

    Ok(jwt_string)
}

fn set_user_credentials(creds: &for_discord_bridge::UserLoginCredentials) -> Result<()> {
    let path = get_credentials_path()?;
    let json = serde_json::to_string(creds)?;
    fs::write(path, json)?;
    log::info!("Stored credentials for {:?}", &creds.email);
    Ok(())
}

#[tauri::command]
async fn store_credentials(creds: for_discord_bridge::UserLoginCredentials) -> Result<(), String> {
    log::debug!("store_credentials");
    set_user_credentials(&creds).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn login_with_stored_credentials() -> Result<for_discord_bridge::JwtString, String> {
    log::debug!("login_with_stored_credentials");
    let creds = get_user_credentials().map_err(|e| e.to_string())?;
    let jwt_string = login(creds).await?;
    log::info!("Logged in with stored credentials.");
    Ok(jwt_string)
}

fn get_user_credentials() -> Result<for_discord_bridge::UserLoginCredentials> {
    let path = get_credentials_path()?;
    let json = fs::read_to_string(path)?;
    let creds: for_discord_bridge::UserLoginCredentials = serde_json::from_str(&json)?;
    log::info!("Retrieved credentials for {:?}", &creds.email);
    Ok(creds)
}

#[tauri::command]
async fn stop_and_clear_credentials(app: tauri::AppHandle) -> Result<(), String> {
    log::debug!("stop_and_clear_credentials");
    abort_background_task(app).await;
    clear_user_credentials().map_err(|e| e.to_string())?;
    Ok(())
}

fn clear_user_credentials() -> Result<()> {
    let path = get_credentials_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    log::info!("Cleared credentials.");
    Ok(())
}

pub fn run() -> Result<()> {
    println!("{}", license::info_text());

    let logs_dir = get_data_dir()?.join("logs");

    let rich_presence_channel: FireAndForgetChannel<DiscordRichPresence> =
        for_discord_bridge::fire_and_forget_channel();
    let updater_status_channel: FireAndForgetChannel<updater::UpdaterStatus> =
        for_discord_bridge::fire_and_forget_channel();

    let logging_plugin = tauri_plugin_log::Builder::default()
        .level(tauri_plugin_log::log::LevelFilter::Debug)
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Webview,
        ))
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Folder {
                path: logs_dir,
                file_name: None,
            },
        ))
        .max_file_size(10 * MEGABYTES)
        .build();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            login,
            store_credentials,
            login_with_stored_credentials,
            stop_and_clear_credentials,
            subscribe_to_bridge_channel,
            initiate_discord_rpc_loop
        ])
        .manage(new_background_tasks_container())
        .manage(rich_presence_channel)
        .manage(updater_status_channel)
        .setup(|app| {
            app.handle().plugin(logging_plugin)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow!(e))
}
