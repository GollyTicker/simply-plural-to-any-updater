// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use clap::Parser;

mod api;
mod auth;
mod config;
mod db;
mod db_constraints;
mod db_secret;
mod discord;
mod fronting_status;
mod jwt;
mod macros;
mod model;
mod setup;
mod simply_plural;
mod updater;
mod updater_loop;
mod updater_manager;
mod vrchat;
mod vrchat_auth;
mod webview;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = setup::CliArgs::parse();

    let app_setup = setup::application_setup(&cli_args).await?;

    api::start_application(app_setup).await
}
