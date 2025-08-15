use anyhow::Result;
use serde::Serialize;

use crate::{
    config::UserConfigForUpdater, discord::DiscordUpdater, simply_plural::Fronter,
    vrchat::VRChatUpdater,
};

#[derive(Clone, Serialize, strum_macros::Display)]
pub enum Platform {
    VRChat,
    Discord,
}

#[derive(Clone, Serialize, strum_macros::Display)]
pub enum UpdaterStatus {
    Inactive,
    Paused,
    Running,
    Error(String),
    Unknown,
}

#[derive(Clone, Serialize)]
pub struct UpdaterState {
    pub updater: Platform,
    pub status: UpdaterStatus,
}

pub enum Updater {
    VRChat(Box<VRChatUpdater>),
    Discord(DiscordUpdater),
}

pub fn implemented_updaters() -> Vec<Platform> {
    vec![Platform::VRChat, Platform::Discord]
}

impl Updater {
    pub fn new(platform: Platform) -> Self {
        match platform {
            Platform::VRChat => Self::VRChat(Box::new(VRChatUpdater::new(platform))),
            Platform::Discord => Self::Discord(DiscordUpdater::new(platform)),
        }
    }

    pub const fn platform(&self) -> Platform {
        match self {
            Self::VRChat(_) => Platform::VRChat,
            Self::Discord(_) => Platform::Discord,
        }
    }

    pub fn status(&self, config: &UserConfigForUpdater) -> UpdaterStatus {
        if self.enabled(config) {
            self.last_operation_error()
                .map_or(UpdaterStatus::Running, |e| UpdaterStatus::Error(e.clone()))
        } else {
            UpdaterStatus::Inactive
        }
    }

    pub fn state(&self, config: &UserConfigForUpdater) -> UpdaterState {
        UpdaterState {
            updater: self.platform(),
            status: self.status(config),
        }
    }

    pub const fn last_operation_error(&self) -> Option<&String> {
        match self {
            Self::VRChat(updater) => updater.last_operation_error.as_ref(),
            Self::Discord(updater) => updater.last_operation_error.as_ref(),
        }
    }

    pub const fn enabled(&self, config: &UserConfigForUpdater) -> bool {
        match self {
            Self::VRChat(_) => config.enable_vrchat,
            Self::Discord(_) => config.enable_discord,
        }
    }

    pub async fn setup(&mut self, config: &UserConfigForUpdater) -> Result<()> {
        match self {
            Self::VRChat(updater) => updater.setup(config).await,
            Self::Discord(updater) => updater.setup(config).await,
        }
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &UserConfigForUpdater,
        fronts: &[Fronter],
    ) -> Result<()> {
        match self {
            Self::VRChat(updater) => updater.update_fronting_status(config, fronts).await,
            Self::Discord(updater) => updater.update_fronting_status(config, fronts).await,
        }
    }
}
