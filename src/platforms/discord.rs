use crate::{
    meta_api::SP2ANY_GITHUB_REPOSITORY_URL,
    plurality, updater,
    users::{self},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum_macros::FromRepr;

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

    #[allow(clippy::unused_async)]
    pub async fn update_fronting_status(
        &self,
        _config: &users::UserConfigForUpdater,
        _fronts: &[plurality::Fronter],
    ) -> Result<()> {
        // fronts are sent to fronter_channel automatically by updater work loop
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServerToBridgeSseMessage {
    // If None, then remove old actvity and show nothing.
    pub discord_rich_presence: Option<DiscordRichPresence>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BridgeToServerSseMessage {
    pub discord_updater_status: updater::UpdaterStatus,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DiscordRichPresence {
    pub activity_type: DiscordActivityType,
    pub status_display_type: DiscordStatusDisplayType,
    pub details: Option<String>,
    pub details_url: Option<String>,
    pub state: Option<String>,
    pub state_url: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub large_image_url: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_url: Option<String>,
    pub small_image_text: Option<String>,
    pub party_current: Option<i32>,
    pub party_max: Option<i32>,
    pub button_label: Option<String>,
    pub button_url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, FromRepr)]
#[repr(u8)]
pub enum DiscordActivityType {
    Playing = 0,
    Listening = 2,
    Watching = 3,
    Custom = 4,
    Competing = 5,
}

#[derive(Clone, Serialize, Deserialize, Debug, FromRepr)]
#[repr(u8)]
pub enum DiscordStatusDisplayType {
    Name = 0,
    State = 1,
    Details = 2,
}

#[allow(clippy::needless_pass_by_value)]
pub fn render_fronts_to_discord_rich_presence(
    fronters: Vec<plurality::Fronter>,
    config: &users::UserConfigForUpdater,
) -> Result<DiscordRichPresence> {
    let short_format = plurality::FrontingFormat {
        max_length: Some(30), // seems to fit often enough without '...' truncation
        cleaning: plurality::CleanForPlatform::NoClean,
        prefix: config.status_prefix.clone(),
        status_if_no_fronters: config.status_no_fronts.clone(),
        truncate_names_to_length_if_status_too_long: config.status_truncate_names_to,
    };
    let short_fronters_string = plurality::format_fronting_status(&short_format, &fronters);

    let long_format = plurality::FrontingFormat {
        max_length: Some(50), // seems to fit often enough without '...' truncation
        ..short_format
    };
    let long_fronters_string = plurality::format_fronting_status(&long_format, &fronters);

    let most_recent_fronting_change: Option<i64> = fronters
        .iter()
        .filter_map(|f| f.start_time)
        .max()
        .map(|dt| dt.timestamp());

    let response = DiscordRichPresence {
        activity_type: DiscordActivityType::Playing,
        status_display_type: DiscordStatusDisplayType::Details,
        details: Some(short_fronters_string),
        details_url: Some(SP2ANY_GITHUB_REPOSITORY_URL.to_owned()), // // future: link to fronting web url
        state: Some(long_fronters_string),
        state_url: None,
        start_time: most_recent_fronting_change,
        end_time: None,        // we can't predict when the fronting will stop
        large_image_url: None, // future: populate these fields.
        large_image_text: None,
        small_image_url: None,
        small_image_text: None,
        party_current: Some(fronters.len().try_into()?),
        party_max: None,
        button_label: None, // future: Some("View Online".to_string()), or maybe instead with a 'What's this?' buttom?
        button_url: None,   // future: link to fronting web url
    };

    Ok(response)
}

// Formatting based on activity type: https://discord.com/developers/docs/events/gateway-events#activity-object-activity-types

// activity type: normal. display as rich presence!
// visible on yourself as well as on others. but the button isn't available for everyone to see
// OR
// let activity_type = ActivityType::Custom; // display as custom status message!
// only visible to yourself when you haven't set a custom status message manually AND when you are not hovering
// over your status on the botom left. You can also not see it on your full bio lol.
// however, it seems to be overshadowed by the normal custom status, if it's manually set by the user! to be noted!
//what about hungstatus? and is the RPC method limited or does it work scalably??? Do I need to have it verified?
// https://discord.com/developers/docs/topics/rpc
// or is this already done by this create?
// NOTE. THIS DOESN'T WORK WITH THE OFFICIAL DISCORD CLIENT! I can offer it, but let users know, that it only works with
// certain modded clients and that there is no guarantee.
