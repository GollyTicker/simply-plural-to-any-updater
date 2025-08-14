use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;

use crate::{config_value, config_value_if};
use serde::{Deserialize, Serialize};

use sp2any_macros::WithOptionDefaults;

#[derive(Default, Debug, Clone, Serialize, Deserialize, WithOptionDefaults)]
pub struct UserConfigDbEntries {
    // None: Use default value, if available
    // Some(x): Use this value
    pub wait_seconds: Option<Duration>,
    pub system_name: Option<String>,
    pub simply_plural_token: Option<String>,
    pub simply_plural_base_url: Option<String>,
    pub enable_discord: Option<bool>,
    pub enable_vrchat: Option<bool>,
    pub discord_base_url: Option<String>,
    pub discord_token: Option<String>,
    pub vrchat_username: Option<String>,
    pub vrchat_password: Option<String>,
    pub vrchat_updater_prefix: Option<String>,
    pub vrchat_updater_no_fronts: Option<String>,
    pub vrchat_updater_truncate_names_to: Option<usize>,
    pub vrchat_cookie: Option<String>,
}

pub fn default_user_db_entries() -> UserConfigDbEntries {
    UserConfigDbEntries {
        vrchat_updater_prefix: Some(String::from("F:")),
        vrchat_updater_no_fronts: Some(String::from("none?")),
        vrchat_updater_truncate_names_to: Some(3),
        discord_base_url: Some(String::from("https://discord.com")),
        simply_plural_base_url: Some(String::from("https://api.apparyllis.com/v1")),
        wait_seconds: Some(Duration::from_secs(60)),
        enable_discord: Some(false),
        enable_vrchat: Some(false),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserConfig {
    pub client: Client,
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

pub fn create_config_with_strong_constraints(
    client: Client,
    db_config: UserConfigDbEntries,
) -> Result<UserConfig> {
    eprintln!("Loading config ...");

    let local_config_with_defaults = db_config.with_option_defaults(default_user_db_entries());

    let enable_discord = config_value!(local_config_with_defaults, enable_discord)?;
    let enable_vrchat = config_value!(local_config_with_defaults, enable_vrchat)?;

    let config = UserConfig {
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
            .unwrap_or_default(),
    };

    if !config.vrchat_username.is_empty() {
        eprintln!(
            "Credentials loaded. VRChat Username is '{}'",
            config.vrchat_username
        );
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn generate_example_json() -> Result<()> {
        let example_config = UserConfigDbEntries {
            simply_plural_token: Some(String::from("simply plural token")),
            discord_token: Some(String::from("discord token")),
            system_name: Some(String::from("Our System")),
            vrchat_username: Some(String::from("vrchat username")),
            vrchat_password: Some(String::from("vrchat password")),
            vrchat_cookie: Some(String::from("automatically set when using vrchat")),
            ..default_user_db_entries()
        };

        let example_json_string = serde_json::to_string_pretty(&example_config)?;

        println!("{example_json_string}");

        Ok(())
    }
}
