use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use futures::stream::StreamExt;
use reqwest_eventsource as sse;
use serde::{Deserialize, Serialize};
use sp2any::users;
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::async_runtime::{JoinHandle, Mutex};
use tauri::Manager;

const DEFAULT_SP2ANY_BASE_URL: &str = "https://sp2any.io";

#[derive(Serialize, Deserialize)]
struct UserCredentials {
    email: String,
    password: String,
}

fn get_credentials_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("io", "sp2any", "sp2any.bridge")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    let data_dir = proj_dirs.data_local_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("credentials.json"))
}

#[tauri::command]
async fn subscribe_to_bridge_channel(
    app: tauri::AppHandle,
    jwt: users::JwtString,
) -> Result<(), String> {
    subscribe_to_bridge_channel_anyhow(app, jwt)
        .await
        .map_err(|e| e.to_string())
}

async fn subscribe_to_bridge_channel_anyhow(
    app: tauri::AppHandle,
    jwt: users::JwtString,
) -> Result<()> {
    let base_url = env::var("SP2ANY_BASE_URL").unwrap_or(DEFAULT_SP2ANY_BASE_URL.to_owned());
    let sse_url = format!("{base_url}/api/user/platform/discord/bridge-events");

    let client = reqwest::Client::new();
    let mut event_source: sse::EventSource = sse::EventSource::new(
        client
            .get(&sse_url)
            .header("Authorization", format!("Bearer {}", jwt.inner)),
    )?;

    let background_task = tauri::async_runtime::spawn(async move {
        eprintln!("Starting SSE listener on {}", sse_url);
        while let Some(event) = event_source.next().await {
            match event {
                Ok(sse::Event::Open) => eprintln!("SSE: Connected."),
                Ok(sse::Event::Message(message)) => {
                    eprintln!(
                        "SSE: Message. id='{}', type='{}', data='{}'",
                        message.id, message.event, message.data
                    );
                }
                Err(err) => eprintln!("SSE: Error: {}…", err),
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
    let state = app.state::<Mutex<Option<JoinHandle<()>>>>();
    let mut locked_task = state.lock().await;
    if let Some(handle) = locked_task.take() {
        handle.abort();
    }
}

#[tauri::command]
async fn login(creds: UserCredentials) -> Result<users::JwtString, String> {
    login_anyhow(creds).await.map_err(|e| e.to_string())
}

async fn login_anyhow(creds: UserCredentials) -> Result<users::JwtString> {
    let login_creds = users::UserLoginCredentials {
        email: creds.email.clone().into(),
        password: users::UserProvidedPassword {
            inner: creds.password.clone(),
        },
    };

    let client = reqwest::Client::new();
    let base_url = env::var("SP2ANY_BASE_URL").unwrap_or(DEFAULT_SP2ANY_BASE_URL.to_owned());
    let login_url = format!("{}{}", base_url, "/api/user/login");

    eprintln!("Attempting login: {login_url} with {}", &creds.email);

    let jwt_string = client
        .post(login_url)
        .json(&login_creds)
        .send()
        .await?
        .error_for_status()?
        .json::<users::JwtString>()
        .await?;

    eprintln!("Login successful for {}", &creds.email);

    Ok(jwt_string)
}

fn set_user_credentials(creds: &UserCredentials) -> Result<()> {
    let path = get_credentials_path()?;
    let json = serde_json::to_string(creds)?;
    fs::write(path, json)?;
    eprintln!("Stored credentials for {}", &creds.email);
    Ok(())
}

#[tauri::command]
async fn store_credentials(creds: UserCredentials) -> Result<(), String> {
    set_user_credentials(&creds).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn login_with_stored_credentials() -> Result<users::JwtString, String> {
    let creds = get_user_credentials().map_err(|e| e.to_string())?;
    let jwt_string = login(creds).await?;
    eprintln!("Logged in with stored credentials.");
    Ok(jwt_string)
}

fn get_user_credentials() -> Result<UserCredentials> {
    let path = get_credentials_path()?;
    let json = fs::read_to_string(path)?;
    let creds: UserCredentials = serde_json::from_str(&json)?;
    eprintln!("Retrieved credentials for {}", &creds.email);
    Ok(creds)
}

#[tauri::command]
async fn stop_and_clear_credentials(app: tauri::AppHandle) -> Result<(), String> {
    abort_background_task(app).await;

    clear_user_credentials().map_err(|e| e.to_string())?;
    Ok(())
}

fn clear_user_credentials() -> Result<()> {
    let path = get_credentials_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    eprintln!("Cleared credentials.");
    Ok(())
}

pub fn run() -> Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            login,
            store_credentials,
            login_with_stored_credentials,
            stop_and_clear_credentials,
            subscribe_to_bridge_channel
        ])
        .manage(new_background_task_container())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow!(e))
}
