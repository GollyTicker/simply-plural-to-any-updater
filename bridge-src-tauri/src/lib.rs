mod discord_bridge;
mod local_storage;
mod never;
mod streaming;

use anyhow::{Result, anyhow};
use futures::stream::StreamExt;
use sp2any::for_discord_bridge;
use sp2any::for_discord_bridge::FireAndForgetChannel;
use sp2any::license;
use sp2any::platforms::ServerToBridgeSseMessage;
use sp2any::updater;
use std::env;
use std::sync::Arc;
use tauri::Emitter;
use tauri::Manager;
use tauri::async_runtime::{JoinHandle, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};

// todo. add auto-update capabilities.
// todo. add auto-start capabilities: https://crates.io/crates/auto-launch
// todo. note, that only a single user account is supported for now.

const MEGABYTES: u128 = 10 ^ 6;

#[tauri::command]
async fn fetch_base_url_and_variant_info()
-> Result<(String, for_discord_bridge::SP2AnyVariantInfo), String> {
    let result = fetch_base_url_and_variant_info_anyhow()
        .await
        .map_err(|e| e.to_string())?;
    Ok(result)
}

async fn fetch_base_url_and_variant_info_anyhow()
-> Result<(String, for_discord_bridge::SP2AnyVariantInfo)> {
    let base_url = local_storage::get_base_url()?;
    let client = reqwest::Client::new();
    let variant_info_url = format!("{}{}", base_url, "/api/meta/sp2any-variant-info");
    let variant_info = client
        .get(&variant_info_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok((base_url, variant_info))
}

#[tauri::command]
async fn initiate_discord_rpc_loop(app: tauri::AppHandle) -> () {
    log::debug!("initiate_discord_rpc_loop");
    let rich_presence_channel = app
        .state::<FireAndForgetChannel<ServerToBridgeSseMessage>>()
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
    let base_url = local_storage::get_base_url()?;
    let ws_url = format!(
        "{}/api/user/platform/discord/bridge-events",
        base_url.replace("http", "ws")
    );

    let mut request = ws_url.clone().into_client_request()?;
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {}", jwt.inner).parse()?);

    log::info!("Connecting to WebSocket at {ws_url}");
    notify_user_on_status(&app, "Connecting to SP2Any to receive updates ...");

    // todo. add retries of connections when they are closed etc.
    // This websocket stream receives text messages of the type DiscordRichPresence and
    // sends messages of the type UpdaterStatus.
    let (ws_stream, _) = connect_async(request).await?;
    let (ws_send, ws_read) = ws_stream.split();

    let forwarder_task = streaming::stream_updater_status_to_ws_messages_task(app.clone(), ws_send);
    register_background_task(app.clone(), forwarder_task).await;

    let receiver_task = streaming::stream_ws_messages_to_rich_presence_task(app.clone(), ws_read);
    register_background_task(app, receiver_task).await;

    Ok(())
}

async fn register_background_task(app: tauri::AppHandle, handle: JoinHandle<()>) {
    let state = app.state::<Arc<Mutex<Vec<JoinHandle<()>>>>>();
    state.lock().await.push(handle);
}

pub fn notify_user_on_status<S: Into<String>>(app: &tauri::AppHandle, value: S) {
    let _ = app.emit("notify_user_on_status", value.into());
    // we don't care about the success.
}

fn new_background_tasks_container() -> Arc<Mutex<Vec<JoinHandle<()>>>> {
    Arc::new(Mutex::new(vec![]))
}

async fn abort_background_task(app: tauri::AppHandle) -> () {
    log::debug!("abort_background_task");
    let state = app.state::<Arc<Mutex<Vec<JoinHandle<()>>>>>();
    let thread_shared_tasks = state.inner().clone();
    let mut locked_tasks = thread_shared_tasks.lock().await;
    for_discord_bridge::blocking_abort_and_clear_tasks(
        &mut locked_tasks,
        |tauri::async_runtime::JoinHandle::Tokio(task)| task,
    );
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
    let base_url = local_storage::get_base_url()?;
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

#[tauri::command]
async fn store_credentials(
    creds: for_discord_bridge::UserLoginCredentials,
    base_url: String,
) -> Result<(), String> {
    log::debug!("store_credentials");
    local_storage::set_base_url(base_url).map_err(|e| e.to_string())?;
    local_storage::set_user_credentials(&creds).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn login_with_stored_credentials() -> Result<for_discord_bridge::JwtString, String> {
    log::debug!("login_with_stored_credentials");
    let creds = local_storage::get_user_credentials().map_err(|e| e.to_string())?;
    let jwt_string = login(creds).await?;
    log::info!("Logged in with stored credentials.");
    Ok(jwt_string)
}
#[tauri::command]
async fn stop_and_clear_credentials(app: tauri::AppHandle) -> Result<(), String> {
    log::debug!("stop_and_clear_credentials");
    abort_background_task(app).await;
    local_storage::clear_user_credentials().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run() -> Result<()> {
    println!("{}", license::info_text());

    let logs_dir = local_storage::get_logs_dir()?;

    let rich_presence_channel: FireAndForgetChannel<ServerToBridgeSseMessage> =
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
            initiate_discord_rpc_loop,
            fetch_base_url_and_variant_info
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
