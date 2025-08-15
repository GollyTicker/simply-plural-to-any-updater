use anyhow::{anyhow, Result};
use reqwest::Client;
use sqlx::FromRow;

use crate::{
    config_value, config_value_if,
    database::WaitSeconds,
    model::{DecryptedDbSecret, SecretType},
};
use serde::{Deserialize, Serialize};

use sp2any_macros::WithOptionDefaults;

#[derive(Default, Debug, Clone, Serialize, Deserialize, WithOptionDefaults, FromRow)]
pub struct UserConfigDbEntries<Secret>
where
    Secret: SecretType,
{
    // None: Use default value, if available
    // Some(x): Use this value
    pub wait_seconds: Option<i32>,

    pub system_name: Option<String>,

    pub status_prefix: Option<String>,
    pub status_no_fronts: Option<String>,
    pub status_truncate_names_to: Option<i32>,

    pub enable_discord: Option<bool>,
    pub enable_vrchat: Option<bool>,

    pub simply_plural_token: Option<Secret>,
    pub discord_token: Option<Secret>,
    pub vrchat_username: Option<Secret>,
    pub vrchat_password: Option<Secret>,
    pub vrchat_cookie: Option<Secret>,

    pub discord_base_url: Option<String>,
    pub simply_plural_base_url: Option<String>,
}

pub fn default_user_db_entries<S: SecretType>() -> UserConfigDbEntries<S> {
    UserConfigDbEntries::<S> {
        status_prefix: Some(String::from("F:")),
        status_no_fronts: Some(String::from("none?")),
        status_truncate_names_to: Some(3),
        discord_base_url: Some(String::from("https://discord.com")),
        simply_plural_base_url: Some(String::from("https://api.apparyllis.com/v1")),
        wait_seconds: Some(60),
        enable_discord: Some(false),
        enable_vrchat: Some(false),
        ..Default::default()
    }
}

#[derive(Clone, Default)]
pub struct UserConfigForUpdater {
    pub client: Client,
    // Note: Keep this in sync with config_store::LocalJsonConfigV2 !
    pub wait_seconds: WaitSeconds,

    pub system_name: String,
    pub status_prefix: String,
    pub status_no_fronts: String,
    pub status_truncate_names_to: usize,

    pub enable_discord: bool,
    pub enable_vrchat: bool,

    pub simply_plural_token: DecryptedDbSecret,
    pub discord_token: DecryptedDbSecret,
    pub vrchat_username: DecryptedDbSecret,
    pub vrchat_password: DecryptedDbSecret,
    pub vrchat_cookie: DecryptedDbSecret,

    pub simply_plural_base_url: String,
    pub discord_base_url: String,
}

pub fn create_config_with_strong_constraints(
    client: Client,
    db_config: UserConfigDbEntries<DecryptedDbSecret>,
) -> Result<UserConfigForUpdater> {
    eprintln!("Loading config ...");

    let local_config_with_defaults = db_config.with_option_defaults(default_user_db_entries());

    let enable_discord = config_value!(local_config_with_defaults, enable_discord)?;
    let enable_vrchat = config_value!(local_config_with_defaults, enable_vrchat)?;

    let config = UserConfigForUpdater {
        client,
        wait_seconds: config_value!(local_config_with_defaults, wait_seconds)?.into(),
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
        status_prefix: config_value!(local_config_with_defaults, status_prefix)?,
        status_no_fronts: config_value!(local_config_with_defaults, status_no_fronts)?,
        status_truncate_names_to: config_value!(
            local_config_with_defaults,
            status_truncate_names_to
        )?
        .try_into()?,
        vrchat_cookie: config_value!(local_config_with_defaults, vrchat_cookie)
            .inspect(|_| eprintln!("A VRChat cookie was found and will be used."))
            .unwrap_or_default(),
    };

    if !config.vrchat_username.secret.is_empty() {
        eprintln!(
            "Credentials loaded. VRChat Username is '{}'",
            config.vrchat_username.secret
        );
    }

    Ok(config)
}
