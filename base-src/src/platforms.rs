use serde::{Deserialize, Serialize};
use strum_macros::FromRepr;

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
