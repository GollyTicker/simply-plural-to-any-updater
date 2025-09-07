use crate::{
    plurality,
    users::{self, UserId},
};
use anyhow::Result;
use serde::Serialize;

pub struct DiscordUpdater {
    pub last_operation_error: Option<String>,
}
impl Default for DiscordUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscordUpdater {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_operation_error: None,
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn setup(&self, _config: &users::UserConfigForUpdater) -> Result<()> {
        Ok(())
    }

    pub async fn update_fronting_status(
        &self,
        _config: &users::UserConfigForUpdater,
        _fronts: &[plurality::Fronter],
    ) -> Result<()> {
        // fronts are send to fronter_channel automatically by updater work loop
        Ok(())
    }
}

#[derive(Serialize)]
pub struct DiscordRichPresence {
    pub details: String,
    pub state: String,
    pub large_image_url: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_url: Option<String>,
    pub small_image_text: Option<String>,
    pub party_current: Option<i32>,
    pub party_max: Option<i32>,
    pub button_label: Option<String>,
    pub button_url: Option<String>,
}

pub async fn render_fronts_to_discord_rich_presence(
    user_id: &UserId,
    fronters: Vec<plurality::Fronter>,
) -> Result<DiscordRichPresence> {
    let fronting_format = plurality::FrontingFormat {
        max_length: None,
        cleaning: plurality::CleanForPlatform::NoClean,
        prefix: "F:".to_owned(), // todo. config.status_prefix.clone(),
        status_if_no_fronters: "none?".to_owned(), // config.status_no_fronts.clone(),
        truncate_names_to_length_if_status_too_long: 1, // todo.
    };
    let status_string = plurality::format_fronting_status(&fronting_format, &fronters);

    let (large_image_url, large_image_text) = if fronters.len() == 1 {
        // todo. put something else here
        (
            Some(fronters[0].avatar_url.clone()),
            Some(fronters[0].name.clone()),
        )
    } else {
        (None, None)
    };

    let response = DiscordRichPresence {
        details: status_string.clone(),
        state: status_string,
        large_image_url,
        large_image_text,
        small_image_url: None,
        small_image_text: None,
        party_current: Some(fronters.len() as i32),
        party_max: None,
        button_label: Some("View Fronters".to_string()),
        button_url: Some(format!("/api/fronting/{user_id}")), //todo.
    };

    Ok(response)
}
