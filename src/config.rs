use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;

use crate::config_store;
use crate::CliArgs;
use crate::{config_value, config_value_if};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub client: Client,
    pub cli_args: CliArgs,
    // Note: Keep this in sync with config_store::LocalJsonConfigV2 !
    pub wait_seconds: Duration,
    pub system_name: String,
    pub simply_plural_token: String,
    pub simply_plural_base_url: String,
    pub enable_discord: bool,
    pub enable_vrchat: bool,
    pub discord_token: String,
    pub discord_base_url: String,
    pub vrchat_username: String,
    pub vrchat_password: String,
    pub vrchat_updater_prefix: String,
    pub vrchat_updater_no_fronts: String,
    pub vrchat_updater_truncate_names_to: usize,
    pub vrchat_cookie: String,
}

pub fn setup_and_load_config(cli_args: &CliArgs) -> Result<Config> {
    eprintln!("Loading config ...");

    let local_config_with_defaults = config_store::read_local_config_file(cli_args)?
        .with_option_defaults(config_store::default_config());

    let request_timeout = config_value!(local_config_with_defaults, request_timeout)?;
    let client = Client::builder()
        .cookie_store(true)
        .timeout(request_timeout)
        .build()?;

    let enable_discord = config_value!(local_config_with_defaults, enable_discord)?;
    let enable_vrchat = config_value!(local_config_with_defaults, enable_vrchat)?;

    let config = Config {
        client,
        wait_seconds: config_value!(local_config_with_defaults, wait_seconds)?,
        system_name: config_value!(local_config_with_defaults, system_name)?,
        simply_plural_token: config_value!(local_config_with_defaults, simply_plural_token)?,
        simply_plural_base_url: config_value!(local_config_with_defaults, simply_plural_base_url)?,
        enable_discord,
        enable_vrchat,
        discord_base_url: config_value_if!(
            enable_discord,
            local_config_with_defaults,
            discord_base_url
        )?,
        discord_token: config_value_if!(enable_discord, local_config_with_defaults, discord_token)?,
        vrchat_username: config_value_if!(
            enable_vrchat,
            local_config_with_defaults,
            vrchat_username
        )?,
        vrchat_password: config_value_if!(
            enable_vrchat,
            local_config_with_defaults,
            vrchat_password
        )?,
        vrchat_updater_prefix: config_value!(local_config_with_defaults, vrchat_updater_prefix)?,
        vrchat_updater_no_fronts: config_value!(
            local_config_with_defaults,
            vrchat_updater_no_fronts
        )?,
        vrchat_updater_truncate_names_to: config_value!(
            local_config_with_defaults,
            vrchat_updater_truncate_names_to
        )?,
        vrchat_cookie: config_value!(local_config_with_defaults, vrchat_cookie)
            .inspect(|_| eprintln!("A VRChat cookie was found and will be used."))
            .unwrap_or(String::new()),
        cli_args: cli_args.clone(),
    };

    if !config.vrchat_username.is_empty() {
        eprintln!(
            "Credentials loaded. VRChat Username is '{}'",
            config.vrchat_username
        );
    }

    Ok(config)
}
