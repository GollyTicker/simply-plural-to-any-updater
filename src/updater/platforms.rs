use anyhow::Result;
use serde::Serialize;
use sp2any_base::updater::UpdaterStatus;
use specta;
use strum_macros;

use crate::{database, platforms, plurality, users};

#[derive(Copy, Clone, Serialize, strum_macros::Display, Eq, Hash, PartialEq, specta::Type)]
#[specta(export)]
pub enum Platform {
    VRChat,
    Discord,
    DiscordStatusMessage,
    ToPluralKit,
}

pub enum Updater {
    VRChat(Box<platforms::VRChatUpdater>),
    Discord(platforms::DiscordUpdater),
    DiscordStatusMessage(platforms::DiscordStatusMessageUpdater),
    ToPluralKit(platforms::ToPluralKitUpdater),
}

#[must_use]
pub fn available_updaters(discord_status_message: bool) -> Vec<Platform> {
    let mut platforms = vec![Platform::VRChat, Platform::Discord, Platform::ToPluralKit];

    if discord_status_message {
        platforms.push(Platform::DiscordStatusMessage);
    }

    for p in platforms.iter().by_ref() {
        log::info!("# | available_updaters | available {p}");
    }

    platforms
}

#[must_use]
pub fn sp2any_server_updaters(discord_status_message: bool) -> Vec<Platform> {
    let mut platforms = available_updaters(discord_status_message);

    platforms.retain(|p| !p.foreign_managed());

    for p in platforms.iter().by_ref() {
        log::info!("# | sp2any_server_updaters | available (managed) {p}");
    }

    platforms
}

impl Platform {
    /// Returns true, if the updating of this target is managed not by the `SP2Any` server.
    #[must_use]
    pub const fn foreign_managed(&self) -> bool {
        match self {
            Self::Discord => true,
            Self::DiscordStatusMessage | Self::VRChat | Self::ToPluralKit => false,
        }
    }
}

#[must_use]
pub const fn initial_status(
    platform: Platform,
    config: &users::UserConfigForUpdater,
) -> UpdaterStatus {
    let enabled = match platform {
        Platform::Discord => config.enable_discord,
        Platform::VRChat => config.enable_vrchat,
        Platform::DiscordStatusMessage => config.enable_discord_status_message,
        Platform::ToPluralKit => config.enable_to_pluralkit,
    };
    if enabled {
        UpdaterStatus::Starting
    } else {
        UpdaterStatus::Disabled
    }
}

impl Updater {
    #[must_use]
    pub fn new(platform: &Platform) -> Self {
        match platform {
            Platform::VRChat => Self::VRChat(Box::default()),
            Platform::Discord => Self::Discord(platforms::DiscordUpdater::new()),
            Platform::DiscordStatusMessage => {
                Self::DiscordStatusMessage(platforms::DiscordStatusMessageUpdater::new())
            }
            Platform::ToPluralKit => Self::ToPluralKit(platforms::ToPluralKitUpdater::new()),
        }
    }

    #[must_use]
    pub const fn platform(&self) -> Platform {
        match self {
            Self::VRChat(_) => Platform::VRChat,
            Self::Discord(_) => Platform::Discord,
            Self::DiscordStatusMessage(_) => Platform::DiscordStatusMessage,
            Self::ToPluralKit(_) => Platform::ToPluralKit,
        }
    }

    #[must_use]
    pub fn status(&self, config: &users::UserConfigForUpdater) -> UpdaterStatus {
        if self.enabled(config) {
            self.last_operation_error()
                .map_or(UpdaterStatus::Running, |e| UpdaterStatus::Error(e.clone()))
        } else {
            UpdaterStatus::Disabled
        }
    }

    const fn last_operation_error(&self) -> Option<&String> {
        match self {
            Self::VRChat(updater) => updater.last_operation_error.as_ref(),
            Self::Discord(updater) => updater.last_operation_error.as_ref(),
            Self::DiscordStatusMessage(updater) => updater.last_operation_error.as_ref(),
            Self::ToPluralKit(updater) => updater.last_operation_error.as_ref(),
        }
    }

    #[must_use]
    pub const fn enabled(&self, config: &users::UserConfigForUpdater) -> bool {
        match self {
            Self::VRChat(_) => config.enable_vrchat,
            Self::Discord(_) => config.enable_discord,
            Self::DiscordStatusMessage(_) => config.enable_discord_status_message,
            Self::ToPluralKit(_) => config.enable_to_pluralkit,
        }
    }

    pub async fn setup(
        &mut self,
        config: &users::UserConfigForUpdater,
        db_pool: &sqlx::PgPool,
        application_user_secrets: &database::ApplicationUserSecrets,
    ) -> Result<()> {
        match self {
            Self::VRChat(updater) => {
                updater
                    .setup(config, db_pool, application_user_secrets)
                    .await
            }
            Self::Discord(updater) => updater.setup(config).await,
            Self::DiscordStatusMessage(updater) => updater.setup(config).await,
            Self::ToPluralKit(updater) => updater.setup(config).await,
        }
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &users::UserConfigForUpdater,
        fronts: &[plurality::Fronter],
    ) -> Result<()> {
        match self {
            Self::VRChat(updater) => updater.update_fronting_status(config, fronts).await,
            Self::Discord(updater) => updater.update_fronting_status(config, fronts).await,
            Self::DiscordStatusMessage(updater) => {
                updater.update_fronting_status(config, fronts).await
            }
            Self::ToPluralKit(updater) => updater.update_fronting_status(config, fronts).await,
        }
    }
}
