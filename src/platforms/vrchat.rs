use crate::plurality;
use crate::record_if_error;
use crate::{platforms::vrchat_auth, users};
use anyhow::anyhow;
use anyhow::{Ok, Result};
use vrchatapi::{
    apis::{configuration::Configuration as VrcConfig, users_api},
    models as vrc,
};

type InitializedUpdater = (VrcConfig, String);
pub struct VRChatUpdater {
    pub last_operation_error: Option<String>,
    initialized: Option<InitializedUpdater>,
}
impl VRChatUpdater {
    pub const fn new() -> Self {
        Self {
            last_operation_error: None,
            initialized: None,
        }
    }

    pub async fn setup(&mut self, config: &users::UserConfigForUpdater) -> Result<()> {
        let init_value = record_if_error!(
            self,
            vrchat_auth::authenticate_vrchat_with_cookie(config).await
        );
        self.initialized = Some(init_value?);
        Ok(())
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &users::UserConfigForUpdater,
        fronts: &[plurality::Fronter],
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
    config: &users::UserConfigForUpdater,
    initialized_updater: &InitializedUpdater,
    fronts: &[plurality::Fronter],
) -> Result<()> {
    let fronting_format = plurality::FrontingFormat {
        max_length: Some(plurality::VRCHAT_MAX_ALLOWED_STATUS_LENGTH),
        cleaning: plurality::CleanForPlatform::VRChat,
        prefix: config.status_prefix.clone(),
        status_if_no_fronters: config.status_no_fronts.clone(),
        truncate_names_to_length_if_status_too_long: config.status_truncate_names_to,
    };

    let status_string = plurality::format_fronting_status(&fronting_format, fronts);

    set_vrchat_status(initialized_updater, status_string.as_str()).await
}

async fn set_vrchat_status(
    initialized_updater: &InitializedUpdater,
    status_string: &str,
) -> Result<()> {
    let mut update_request = vrc::UpdateUserRequest::new();
    update_request.status_description = Some(status_string.to_string());

    let (vrchat_config, user_id) = initialized_updater;
    users_api::update_user(vrchat_config, user_id, Some(update_request))
        .await
        .inspect(|_| eprintln!("VRChat status updated successfully to: '{status_string}'"))?;

    Ok(())
}
