#[macro_use]
extern crate rocket;

mod config;
mod simply_plural;
mod vrchat;
mod vrchat_auth;
mod vrchat_status;
mod webserver;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result};
use tokio::sync::mpsc;

// Helper macro for logging to console and optionally to GUI
macro_rules! app_log {
    ($log_sender:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        eprintln!("{}", msg); // Always print to console
        if let Some(sender) = $log_sender {
            let sender = sender.clone();
            // Spawn a task to send the message without blocking the current flow
            tokio::spawn(async move {
                if sender.send(msg).await.is_err() {
                    // If sending to GUI fails, print an error to the actual stderr.
                    // Avoid using app_log! here to prevent potential recursion.
                    std::eprintln!("Failed to send log message to GUI channel.");
                }
            });
        }
    }
}

async fn run_app_logic(log_sender_option: Option<mpsc::Sender<String>>) -> Result<()> {
    app_log!(&log_sender_option, "Starting Simply Plural to Any Updater...");

    let config = match config::setup_and_load_config().await {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to load config: {:?}", e);
            app_log!(&log_sender_option, "{}", err_msg);
            return Err(e.context(err_msg));
        }
    };

    if config.serve_api {
        app_log!(&log_sender_option, "Running in Webserver mode.");
        if let Err(e) = webserver::run_server(&config).await {
            let err_msg = format!("Webserver error: {:?}", e);
            app_log!(&log_sender_option, "{}", err_msg);
            return Err(e.context(err_msg));
        }
    } else {
        app_log!(&log_sender_option, "Running in VRChat Updater mode.");
        if let Err(e) = vrchat::run_updater_loop(&config).await {
            let err_msg = format!("VRChat updater error: {:?}", e);
            app_log!(&log_sender_option, "{}", err_msg);
            return Err(e.context(err_msg));
        }
    }
    app_log!(&log_sender_option, "Application logic completed successfully.");
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let no_gui_arg = args.contains(&"--no-gui".to_string());
    let no_gui_env = std::env::var("NO_GUI").map_or(false, |val| val.eq_ignore_ascii_case("true"));
    let no_gui = no_gui_arg || no_gui_env;

    if no_gui {
        eprintln!("Running in NO_GUI mode.");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            run_app_logic(None).await // Pass None for the log_sender
        })?;
    } else {
        let (log_tx, mut log_rx) = mpsc::channel::<String>(100); // Channel for log messages

        tauri::Builder::default()
            .setup(move |app| {
                let app_handle = app.handle();
                let core_logic_log_tx = log_tx.clone();

                // Spawn the core application logic
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = run_app_logic(Some(core_logic_log_tx.clone())).await {
                        let final_error_msg = format!("Core application exited with error: {:?}", e);
                        eprintln!("{}", final_error_msg); // Log to console
                        // Attempt to send this final status to GUI as well
                        let _ = core_logic_log_tx.send(final_error_msg).await;
                    } else {
                        let success_msg = "Core application logic finished.".to_string();
                        eprintln!("{}", success_msg);
                        let _ = core_logic_log_tx.send(success_msg).await;
                    }
                });

                // Spawn the log message forwarder to Tauri frontend
                tauri::async_runtime::spawn(async move {
                    while let Some(message) = log_rx.recv().await {
                        app_handle.emit_all("log-message", &message).unwrap_or_else(|e| {
                            std::eprintln!("Failed to emit log to frontend: {}", e);
                        });
                    }
                });
                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
    Ok(())
}
