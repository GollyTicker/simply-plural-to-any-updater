use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;

use crate::CliArgs;
use sp2any_macros::WithOptionDefaults;

#[derive(Default, Debug, Clone, Serialize, Deserialize, WithOptionDefaults)]
pub struct LocalJsonConfigV2 {
    // None: Use default value from github, if available
    // Some(x): Use this value
    pub wait_seconds: Option<Duration>,
    pub request_timeout: Option<Duration>,
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

pub fn default_config() -> LocalJsonConfigV2 {
    LocalJsonConfigV2 {
        vrchat_updater_prefix: Some(String::from("F:")),
        vrchat_updater_no_fronts: Some(String::from("none?")),
        vrchat_updater_truncate_names_to: Some(3),
        discord_base_url: Some(String::from("https://discord.com")),
        simply_plural_base_url: Some(String::from("https://api.apparyllis.com/v1")),
        wait_seconds: Some(Duration::from_secs(60)),
        request_timeout: Some(Duration::from_secs(5)),
        enable_discord: Some(false),
        enable_vrchat: Some(false),
        ..Default::default()
    }
}

fn local_json_config_file_path(operation: &str, cli_args: &CliArgs) -> Result<String> {
    let file_path = if cli_args.config.clone().is_empty() {
        #[allow(clippy::unwrap_used)]
        let project_dir = ProjectDirs::from("org", "sp2any", "sp2any").unwrap();
        project_dir
            .config_dir()
            .join("sp2any.json")
            .to_str()
            .map(String::from)
            .ok_or_else(|| anyhow!("Path to String conversion failed"))
    } else {
        Ok(cli_args.config.clone())
    }?;

    eprintln!("Local JSON Config file ({operation}): {file_path:?}");

    Ok(file_path)
}

fn check_local_config_file_exists(cli_args: &CliArgs) -> Result<bool> {
    let config_file_path = local_json_config_file_path("check", cli_args)?;
    let exists = fs::exists(config_file_path)?;
    Ok(exists)
}

pub fn read_local_config_file(cli_args: &CliArgs) -> Result<LocalJsonConfigV2> {
    let config_file_path = local_json_config_file_path("read", cli_args)?;
    let config_as_string = fs::read_to_string(config_file_path)?;
    let local_config = serde_json::from_str(config_as_string.as_str())?;
    Ok(local_config)
}

pub fn write_local_config_file(local_config: &LocalJsonConfigV2, cli_args: &CliArgs) -> Result<()> {
    let config_file_path = local_json_config_file_path("write", cli_args)?;
    let config_as_string = serde_json::to_string_pretty(local_config)?;
    fs::write(config_file_path, config_as_string)?;
    Ok(())
}

pub fn store_vrchat_cookie(cookie_str: &str, cli_args: &CliArgs) -> Result<()> {
    let mut local_config = read_local_config_file(cli_args)?;
    local_config.vrchat_cookie = Some(cookie_str.to_owned());
    write_local_config_file(&local_config, cli_args)?;
    eprintln!("VRChat cookie stored.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn generate_example_json() -> Result<()> {
        let example_config = LocalJsonConfigV2 {
            simply_plural_token: Some(String::from("simply plural token")),
            discord_token: Some(String::from("discord token")),
            system_name: Some(String::from("Our System")),
            vrchat_username: Some(String::from("vrchat username")),
            vrchat_password: Some(String::from("vrchat password")),
            vrchat_cookie: Some(String::from("automatically set when using vrchat")),
            ..default_config()
        };

        let example_json_string = serde_json::to_string_pretty(&example_config)?;
        fs::write("release/config/example.json", example_json_string)?;

        Ok(())
    }
}
