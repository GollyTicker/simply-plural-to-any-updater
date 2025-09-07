use crate::{
    plurality,
    users::{self, UserId},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

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

#[allow(clippy::needless_pass_by_value)]
pub fn render_fronts_to_discord_rich_presence(
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

// const FRONTING_TEST_IMAGE: &str = "https://ayake.net/cloud/apps/files_sharing/publicpreview/wewER2MaZ4JbXEg?file=/&fileId=28035&x=3424&y=1926&a=true&etag=d150d19707ca3b6ef1470e0853bb7da7";
// fn source() {
//     let activity_type = ActivityType::Playing; // display as rich presence!
//                                                // visible on yourself as well as on others. but the button isn't available for everyone to see
//                                                // OR
//                                                // let activity_type = ActivityType::Custom; // display as custom status message!
//                                                // only visible to yourself when you haven't set a custom status message manually AND when you are not hovering
//                                                // over your status on the botom left. You can also not see it on your full bio lol.
//                                                // however, it seems to be overshadowed by the normal custom status, if it's manually set by the user! to be noted!
//                                                //what about hungstatus? and is the RPC method limited or does it work scalably??? Do I need to have it verified?
//                                                // https://discord.com/developers/docs/topics/rpc
//                                                // or is this already done by this create?
//                                                // NOTE. THIS DOESN'T WORK WITH THE OFFICIAL DISCORD CLIENT! I can offer it, but let users know, that it only works with
//                                                // certain modded clients and that there is no guarantee.

//     // Formatting based on activity type: https://discord.com/developers/docs/events/gateway-events#activity-object-activity-types

//     let payload = activity::Activity::new()
//         .activity_type(activity_type)
//         .timestamps(
//             Timestamps::new()
//                 .start(Utc::now().timestamp() - 1000)
//                 .end(Utc::now().timestamp() + 1000),
//         )
//         .details("details: test F: Ayake, Felina, Hole")
//         .state("state: test F: Ayake, Felina, Hole")
//         .party(Party::new().id("party-id").size([3, 9]))
//         .buttons(vec![Button::new(
//             "View Online",
//             "https://ayake.net/fronting",
//         )]) // todo. maybe add a buttom to the fronting website?
//         // .secrets(Secrets::new().spectate("some-sepctate-secret"))
//         .assets(
//             Assets::new()
//                 .small_image(FRONTING_TEST_IMAGE)
//                 .small_text("small Ayake Sparkle 💖")
//                 .large_image(FRONTING_TEST_IMAGE)
//                 .large_text("large Ayake Sparkle 💖"),
//         );
// }
