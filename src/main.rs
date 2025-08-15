// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, PgPool};

mod api;
mod auth;
mod config;
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
mod webview;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    let (db_pool, client) = setup(&cli_args).await?;

    api::run_server(cli_args, client, db_pool).await
}

async fn setup(cli_args: &CliArgs) -> Result<(PgPool, Client)> {
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&cli_args.database_url)
        .await?;

    let client: Client = Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(cli_args.request_timeout))
        .build()?;

    Ok((db_pool, client))
}

#[derive(Parser, Debug, Clone, Default)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long)]
    pub database_url: String,

    #[arg(short, long, default_value_t = 5)]
    pub request_timeout: u64,

    #[arg(short, long)]
    pub jwt_application_secret: String,
}
