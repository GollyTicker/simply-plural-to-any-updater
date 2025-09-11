use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{platforms, plurality, users};

// NOTE: specta::Type is manually exported in bindings
#[derive(Clone, Serialize, strum_macros::Display, Eq, Hash, PartialEq)]
pub enum Platform {
    VRChat,
    Discord,
    DiscordStatusMessage,
}

// NOTE: specta::Type is manually exported in bindings
#[derive(Clone, Serialize, Deserialize, strum_macros::Display)]
pub enum UpdaterStatus {
    Inactive,
    Running,
    Error(String),
}

pub enum Updater {
    VRChat(Box<platforms::VRChatUpdater>),
    Discord(platforms::DiscordUpdater),
    DiscordStatusMessage(platforms::DiscordStatusMessageUpdater),
}

#[must_use]
pub fn sp2any_server_updaters(discord_status_message: bool) -> Vec<Platform> {
    let mut platforms = vec![Platform::VRChat];

    if discord_status_message {
        platforms.push(Platform::DiscordStatusMessage);
    }

    for p in platforms.iter().by_ref() {
        let foreign_managed_text = if p.foreign_managed() {
            "(foreign managed; hence excluded)"
        } else {
            ""
        };
        eprintln!("Platform: {p} {foreign_managed_text}");
    }

    platforms.retain(|p| !p.foreign_managed());

    platforms
}

impl Platform {
    /// Returns true, if the updating of this target is managed not by the `SP2Any` server.
    #[must_use]
    pub const fn foreign_managed(&self) -> bool {
        match self {
            Self::Discord => true,
            Self::DiscordStatusMessage | Self::VRChat => false,
        }
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
        }
    }

    #[must_use]
    pub const fn platform(&self) -> Platform {
        match self {
            Self::VRChat(_) => Platform::VRChat,
            Self::Discord(_) => Platform::Discord,
            Self::DiscordStatusMessage(_) => Platform::DiscordStatusMessage,
        }
    }

    #[must_use]
    pub fn status(&self, config: &users::UserConfigForUpdater) -> UpdaterStatus {
        if self.enabled(config) {
            self.last_operation_error()
                .map_or(UpdaterStatus::Running, |e| UpdaterStatus::Error(e.clone()))
        } else {
            UpdaterStatus::Inactive
        }
    }

    const fn last_operation_error(&self) -> Option<&String> {
        match self {
            Self::VRChat(updater) => updater.last_operation_error.as_ref(),
            Self::Discord(updater) => updater.last_operation_error.as_ref(),
            Self::DiscordStatusMessage(updater) => updater.last_operation_error.as_ref(),
        }
    }

    #[must_use]
    pub const fn enabled(&self, config: &users::UserConfigForUpdater) -> bool {
        match self {
            Self::VRChat(_) => config.enable_vrchat,
            Self::Discord(_) => config.enable_discord,
            Self::DiscordStatusMessage(_) => config.enable_discord_status_message,
        }
    }

    pub async fn setup(&mut self, config: &users::UserConfigForUpdater) -> Result<()> {
        match self {
            Self::VRChat(updater) => updater.setup(config).await,
            Self::Discord(updater) => updater.setup(config).await,
            Self::DiscordStatusMessage(updater) => updater.setup(config).await,
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
        }
    }
}
