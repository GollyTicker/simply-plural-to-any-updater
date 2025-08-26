use anyhow::{anyhow, Result};
use sqlx::FromRow;
use std::time::Duration;

use crate::{config_value, config_value_if, database, users::model::UserId};
use serde::{Deserialize, Serialize};

use sp2any_macros::WithOptionDefaults;

#[derive(
    Default, Debug, Clone, Serialize, Deserialize, WithOptionDefaults, FromRow, PartialEq, Eq,
)]
pub struct UserConfigDbEntries<Secret, Constraints = database::InvalidConstraints>
where
    Secret: database::SecretType,
    Constraints: database::ConstraintsType,
{
    #[serde(skip)]
    pub valid_constraints: Option<Constraints>,

    // None: Use default value, if available
    // Some(x): Use this value
    pub wait_seconds: Option<i32>,

    pub system_name: Option<String>,

    pub status_prefix: Option<String>,
    pub status_no_fronts: Option<String>,
    pub status_truncate_names_to: Option<i32>,

    pub enable_discord: Option<bool>,
    pub enable_discord_status_message: Option<bool>,
    pub enable_vrchat: Option<bool>,

    pub simply_plural_token: Option<Secret>,
    pub discord_status_message_token: Option<Secret>,
    pub discord_user_id: Option<Secret>,
    pub discord_oauth_access_token: Option<Secret>,
    pub discord_oauth_refresh_token: Option<Secret>,
    pub vrchat_username: Option<Secret>,
    pub vrchat_password: Option<Secret>,
    pub vrchat_cookie: Option<Secret>,
}

pub fn default_user_db_entries<S: database::SecretType>() -> UserConfigDbEntries<S> {
    UserConfigDbEntries::<S> {
        status_prefix: Some(String::from("F:")),
        status_no_fronts: Some(String::from("none?")),
        status_truncate_names_to: Some(3),
        wait_seconds: Some(60),
        enable_discord: Some(false),
        enable_discord_status_message: Some(false),
        enable_vrchat: Some(false),
        ..Default::default()
    }
}

pub struct UserConfigForUpdater {
    pub client: reqwest::Client,
    pub user_id: UserId,
    pub simply_plural_base_url: String,
    pub discord_base_url: String,

    // Note: v Keep this in sync with UserConfigDbEntries! v
    pub wait_seconds: WaitSeconds,

    pub system_name: String,
    pub status_prefix: String,
    pub status_no_fronts: String,
    pub status_truncate_names_to: usize,

    pub enable_discord: bool,
    pub enable_discord_status_message: bool,
    pub enable_vrchat: bool,

    pub simply_plural_token: database::Decrypted,
    pub discord_status_message_token: database::Decrypted,
    pub discord_user_id: database::Decrypted,
    pub discord_oauth_access_token: database::Decrypted,
    pub discord_oauth_refresh_token: database::Decrypted,
    pub vrchat_username: database::Decrypted,
    pub vrchat_password: database::Decrypted,
    pub vrchat_cookie: database::Decrypted,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct WaitSeconds {
    pub inner: Duration,
}

impl From<Duration> for WaitSeconds {
    fn from(value: Duration) -> Self {
        Self { inner: value }
    }
}

impl From<i32> for WaitSeconds {
    #[allow(clippy::cast_sign_loss)]
    fn from(secs: i32) -> Self {
        Duration::from_secs(secs as u64).into()
    }
}

pub fn create_config_with_strong_constraints<Constraints>(
    user_id: &UserId,
    client: &reqwest::Client,
    db_config: &UserConfigDbEntries<database::Decrypted, Constraints>,
) -> Result<(
    UserConfigForUpdater,
    UserConfigDbEntries<database::Decrypted, database::ValidConstraints>,
)>
where
    Constraints: database::ConstraintsType,
{
    eprintln!("Loading config ...");

    let db_config = database::downgrade(db_config);
    let local_config_with_defaults = db_config.with_option_defaults(default_user_db_entries());

    let enable_discord = config_value!(local_config_with_defaults, enable_discord)?;
    let enable_discord_status_message =
        config_value!(local_config_with_defaults, enable_discord_status_message)?;
    let enable_vrchat = config_value!(local_config_with_defaults, enable_vrchat)?;

    let config = UserConfigForUpdater {
        user_id: user_id.clone(),
        client: client.clone(),
        wait_seconds: config_value!(local_config_with_defaults, wait_seconds)?.into(),
        system_name: config_value!(local_config_with_defaults, system_name)?,
        simply_plural_token: config_value!(local_config_with_defaults, simply_plural_token)?,
        simply_plural_base_url: String::from("https://api.apparyllis.com/v1"),
        enable_discord,
        enable_discord_status_message,
        enable_vrchat,
        discord_base_url: if enable_discord_status_message {
            String::from("https://discord.com")
        } else {
            String::new()
        },
        discord_status_message_token: config_value_if!(
            enable_discord_status_message,
            local_config_with_defaults,
            discord_status_message_token
        )?,
        discord_user_id: config_value_if!(
            enable_discord,
            local_config_with_defaults,
            discord_user_id
        )?,
        discord_oauth_access_token: config_value_if!(
            enable_discord,
            local_config_with_defaults,
            discord_oauth_access_token
        )?,
        discord_oauth_refresh_token: config_value_if!(
            enable_discord,
            local_config_with_defaults,
            discord_oauth_refresh_token
        )?,
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

    let valid_config =
        database::only_use_this_function_to_mark_validation_after_you_have_actually_validated_it(
            &db_config,
        );

    Ok((config, valid_config))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::Decrypted;

    #[test]
    fn test_user_config_db_entries_serialization() {
        let config = UserConfigDbEntries::<Decrypted> {
            wait_seconds: Some(30),
            system_name: Some("My System".to_string()),
            status_prefix: Some("SP:".to_string()),
            status_no_fronts: Some("No one fronting".to_string()),
            status_truncate_names_to: Some(5),
            enable_discord: Some(true),
            enable_discord_status_message: Some(true),
            enable_vrchat: Some(false),
            simply_plural_token: Some(Decrypted {
                secret: "sp_token_123".to_string(),
            }),
            discord_status_message_token: Some(Decrypted {
                secret: "discord_status_message_token_abc".to_string(),
            }),
            discord_user_id: Some(Decrypted {
                secret: "discord_user_id".to_string(),
            }),
            discord_oauth_access_token: Some(Decrypted {
                secret: "discord_oauth_access_token".to_string(),
            }),
            discord_oauth_refresh_token: Some(Decrypted {
                secret: "discord_oauth_refresh_token".to_string(),
            }),
            discord_pairing_code: Some(Decrypted {
                secret: "123456789".to_string(),
            }),
            discord_bridge_secret: Some(Decrypted {
                secret: "discord_bridge_secret".to_string(),
            }),
            vrchat_username: None,
            vrchat_password: None,
            vrchat_cookie: None,
            valid_constraints: None,
        };

        let json_string = serde_json::to_string_pretty(&config).unwrap();
        let expected_json = r#"{
  "wait_seconds": 30,
  "system_name": "My System",
  "status_prefix": "SP:",
  "status_no_fronts": "No one fronting",
  "status_truncate_names_to": 5,
  "enable_discord": true,
  "enable_discord_status_message": true,
  "enable_vrchat": false,
  "discord_pairing_code_expires_at": null,
  "simply_plural_token": {
    "secret": "sp_token_123"
  },
  "discord_status_message_token": {
    "secret": "discord_status_message_token_abc"
  },
  "discord_user_id": {
    "secret": "discord_user_id"
  },
  "discord_oauth_access_token": {
    "secret": "discord_oauth_access_token"
  },
  "discord_oauth_refresh_token": {
    "secret": "discord_oauth_refresh_token"
  },
  "vrchat_username": null,
  "vrchat_password": null,
  "vrchat_cookie": null
}"#;

        assert_eq!(json_string, expected_json);
    }
}
