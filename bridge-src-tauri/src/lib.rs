mod discord_bridge;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use futures::stream::StreamExt;
use reqwest_eventsource as sse;
use sp2any::for_discord_bridge;
use sp2any::license;
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::async_runtime::{JoinHandle, Mutex};
use tauri::Manager;
use tokio::sync::broadcast;

// todo. add auto-update capabilities.
// todo. add auto-start capabilities: https://crates.io/crates/auto-launch
// todo. note, that only a single user account is supported for now.

const DEFAULT_SP2ANY_BASE_URL: &str = "https://sp2any.io";
const MEGABYTES: u128 = 10^6;

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
    let channel = app
        .state::<broadcast::Sender<for_discord_bridge::DiscordRichPresence>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn(async move {
        discord_bridge::discord_ipc_loop(channel).await;
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
    let sse_url = format!("{base_url}/api/user/platform/discord/bridge-events");

    log::info!("Subscribing to SSE at {sse_url}");

    let client = reqwest::Client::new();
    let mut event_source: sse::EventSource = sse::EventSource::new(
        client
            .get(&sse_url)
            .header("Authorization", format!("Bearer {}", jwt.inner)),
    )?;

    let app2 = app.clone();
    let background_task = tauri::async_runtime::spawn(async move {
        let sender = app2.state::<broadcast::Sender<for_discord_bridge::DiscordRichPresence>>();
        log::info!("Starting SSE listener on {sse_url}");
        while let Some(event) = event_source.next().await {
            match event {
                Ok(sse::Event::Open) => log::info!("SSE: Connected."),
                Ok(sse::Event::Message(message)) => {
                    log::info!("SSE: Message: '{}'", message.data);
                    if let Ok(rich_presence) = serde_json::from_str(&message.data) {
                        let _ = sender.send(rich_presence);
                    }
                }
                Err(err) => log::warn!("SSE: Error: {err}..."),
            }
        }
    });

    register_background_task(app, background_task).await;

    Ok(())
}

async fn register_background_task(app: tauri::AppHandle, handle: JoinHandle<()>) {
    let state = app.state::<Mutex<Option<JoinHandle<()>>>>();
    *state.lock().await = Some(handle);
}

fn new_background_task_container() -> Mutex<Option<JoinHandle<()>>> {
    Mutex::new(Option::<JoinHandle<()>>::None)
}

async fn abort_background_task(app: tauri::AppHandle) -> () {
    log::debug!("abort_background_task");
    let state = app.state::<Mutex<Option<JoinHandle<()>>>>();
    let mut locked_task = state.lock().await;
    if let Some(handle) = locked_task.take() {
        handle.abort();
    }
}

#[tauri::command]
async fn login(creds: for_discord_bridge::UserLoginCredentials) -> Result<for_discord_bridge::JwtString, String> {
    log::debug!("login");
    login_anyhow(creds).await.map_err(|e| e.to_string())
}

async fn login_anyhow(creds: for_discord_bridge::UserLoginCredentials) -> Result<for_discord_bridge::JwtString> {
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
        .manage(new_background_task_container())
        .manage(broadcast::channel::<for_discord_bridge::DiscordRichPresence>(1).0)
        .setup(|app| {
            app.handle().plugin(logging_plugin)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow!(e))
}
