// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use clap::Parser;
use std::future::Future;
use tokio::runtime;

mod config;
mod config_store;
mod discord;
mod fronting_status;
mod macros;
mod simply_plural;
mod updater;
mod vrchat;
mod vrchat_auth;
mod webserver;

fn main() -> Result<()> {
    let cli_args = config_store::CliArgs::parse();

    let config = config::setup_and_load_config(&cli_args)?;

    if cli_args.webserver {
        eprintln!("Running in Webserver mode ...");
        run_async_blocking(webserver::run_server(config))?;
    } else if cli_args.no_gui {
        eprintln!("Running SP2Any Updater in console mode ...");
        run_async_blocking(updater::run_loop(&config))?;
    } else {
        eprintln!("Starting SP2Any Updater in GUI mode ...");
        unimplemented!();
    }

    Ok(())
}

fn run_async_blocking<T>(f: impl Future<Output = Result<T>>) -> Result<T> {
    let rt = runtime::Runtime::new()?;
    rt.block_on(f)
}
