use crate::config::UserConfigForUpdater;
use crate::fronting_status;
use crate::record_if_error;
use crate::simply_plural;
use crate::updater::Platform;
use crate::vrchat_auth;
use anyhow::anyhow;
use anyhow::{Ok, Result};
use vrchatapi::{
    apis::{configuration::Configuration, users_api},
    models::UpdateUserRequest,
};

type InitializedUpdater = (Configuration, String);
pub struct VRChatUpdater {
    pub last_operation_error: Option<String>,
    initialized: Option<InitializedUpdater>,
}
impl VRChatUpdater {
    pub const fn new(_platform: Platform) -> Self {
        Self {
            last_operation_error: None,
            initialized: None,
        }
    }

    pub async fn setup(&mut self, config: &UserConfigForUpdater) -> Result<()> {
        let init_value = record_if_error!(self, vrchat_auth::authenticate_vrchat(config).await);
        self.initialized = Some(init_value?);
        Ok(())
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &UserConfigForUpdater,
        fronts: &[simply_plural::Fronter],
    ) -> Result<()> {
        let initialized_updater = record_if_error!(
            self,
            self.initialized
                .as_ref()
                .ok_or_else(|| anyhow!("Updater not initalized!"))
        );
        record_if_error!(
            self,
            update_to_vrchat(config, initialized_updater?, fronts).await
        )
    }
}

async fn update_to_vrchat(
    config: &UserConfigForUpdater,
    initialized_updater: &InitializedUpdater,
    fronts: &[simply_plural::Fronter],
) -> Result<()> {
    let fronting_format = fronting_status::FrontingFormat {
        max_length: Some(fronting_status::VRCHAT_MAX_ALLOWED_STATUS_LENGTH),
        cleaning: fronting_status::CleanForPlatform::VRChat,
        prefix: config.status_prefix.clone(),
        status_if_no_fronters: config.status_no_fronts.clone(),
        truncate_names_to_length_if_status_too_long: config.status_truncate_names_to,
    };

    let status_string = fronting_status::format_fronting_status(&fronting_format, fronts);

    set_vrchat_status(initialized_updater, status_string.as_str()).await
}

async fn set_vrchat_status(
    initialized_updater: &InitializedUpdater,
    status_string: &str,
) -> Result<()> {
    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.to_string());

    let (vrchat_config, user_id) = initialized_updater;
    users_api::update_user(vrchat_config, user_id, Some(update_request))
        .await
        .inspect(|_| eprintln!("VRChat status updated successfully to: '{status_string}'"))?;

    Ok(())
}
