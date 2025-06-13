#[macro_use]
extern crate rocket;

mod config;
mod simply_plural;
mod vrchat;
mod vrchat_auth;
mod vrchat_status;
mod webserver;

use anyhow::Result;
use clap::Parser;
use tokio::runtime;
use tracing_subscriber::{self};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Run without the graphical user interface
    #[clap(long, env = "NO_GUI", action = clap::ArgAction::SetTrue)]
    no_gui: bool,
}

async fn run_app_logic() -> Result<()> {
    tracing::info!("Starting Simply Plural to Any Updater ...");

    let config = config::setup_and_load_config().await?;

    if config.serve_api {
        tracing::info!("Running in Webserver mode.");
        webserver::run_server(&config).await
    } else {
        tracing::info!("Running in VRChat Updater mode.");
        vrchat::run_updater_loop(&config).await
    }
}

/// Sets up and runs the Tauri application.
fn run_tauri_application() -> Result<()> {
    tracing::info!("GUI mode selected. Initializing Tauri application...");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            tracing::info!("Tauri application setup complete. Spawning core logic...");

            // Spawn the core application logic
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_app_logic().await {
                    tracing::error!("Core application error: {:?}", e);
                } else {
                    tracing::info!("Core application finished successfully.");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!()) // generate_context! should work with Tauri.toml in v2
        .map_err(anyhow::Error::from)
}

fn main() -> Result<()> {
    let cli_args = Cli::parse();

    if cli_args.no_gui {
        no_gui_mode_tracing_setup();
        tracing::info!("No-GUI mode selected. Running in console.");
        runtime::Runtime::new()
            .unwrap()
            .block_on(run_app_logic())
    } else {
        eprintln!("Start in GUI mode...");
        run_tauri_application()
    }
}

fn no_gui_mode_tracing_setup() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(true)
        .init();
}
