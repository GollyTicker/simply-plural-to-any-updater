#[macro_use]
extern crate rocket;

mod database;
mod http;
pub mod platforms;
mod plurality;
pub mod setup;
pub mod updater;
pub mod users;

pub mod for_discord_bridge {
    pub use crate::platforms::DiscordRichPresence;
    pub use crate::users::user_api::UserLoginCredentials;
    pub use crate::users::{JwtString, UserProvidedPassword};
}
