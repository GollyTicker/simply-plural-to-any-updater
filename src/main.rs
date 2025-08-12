// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

mod auth;
mod config;
mod config_store;
mod database;
mod discord;
mod fronting_status;
mod macros;
mod model;
mod simply_plural;
mod updater;
mod updater_loop;
mod vrchat;
mod vrchat_auth;
mod webserver;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    let config = config::setup_and_load_config(&cli_args)?;

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cli_args.database_url)
        .await?;

    webserver::run_server(cli_args, config, db_pool).await?;

    Ok(())
}

#[derive(Parser, Debug, Clone, Default)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    // Path to local json config file, if not default
    #[arg(short, long, default_value_t = String::new())]
    pub config: String,

    // Database URL
    #[arg(long, default_value_t = String::from("postgres://postgres:postgres@localhost/sp2any"))]
    pub database_url: String,
}
