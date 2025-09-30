#[macro_use]
extern crate rocket;

mod communication;
pub mod database;
pub mod meta_api;
pub mod metrics;
pub mod platforms;
pub mod plurality;
pub mod setup;
pub mod updater;
pub mod users;

pub mod for_discord_bridge {
    pub use crate::communication::{
        FireAndForgetChannel, LatestReceiver, blocking_abort_and_clear_tasks,
        fire_and_forget_channel,
    };
    pub use crate::meta_api::{CANONICAL_SP2ANY_BASE_URL, SP2AnyVariantInfo};
    pub use crate::platforms::DiscordRichPresence;
    pub use crate::users::user_api::UserLoginCredentials;
    pub use crate::users::{JwtString, UserProvidedPassword};
}

pub mod license {
    #[must_use]
    pub fn info_text() -> String {
        format!(
            "==========\n{}\n==========",
            include_str!("../docker/license-info.txt")
        )
    }
    #[must_use]
    pub fn info_short_html() -> String {
        include_str!("../docker/license-info-short.html").to_owned()
    }
}
