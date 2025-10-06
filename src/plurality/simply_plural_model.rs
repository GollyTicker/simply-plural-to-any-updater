use std::string::ToString;

use anyhow::Result;
use serde::Deserialize;
use serde::Deserializer;

pub const GLOBAL_SP2ANY_ON_SIMPLY_PLURAL_USER_ID: &str =
    "eb06960e5b7fb576923f0e909947c0ce8ca46dcbe61ee5af2681f8f59404df5d";

pub const SIMPLY_PLURAL_VRCHAT_STATUS_NAME_FIELD_NAME: &str = "VRChat Status Name";

#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntry {
    pub content: FrontEntryContent,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntryContent {
    /** Can be a member ID OR a custom front ID */
    #[serde(rename = "member")]
    pub fronter_id: String,

    #[serde(rename = "uid")]
    pub system_id: String,

    #[serde(rename = "startTime")]
    #[serde(deserialize_with = "parse_epoch_millis_to_datetime_utc")]
    pub start_time: chrono::DateTime<chrono::Utc>,
}

fn parse_epoch_millis_to_datetime_utc<'de, D>(
    d: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let epoch_millis = i64::deserialize(d)?;
    chrono::DateTime::from_timestamp_millis(epoch_millis)
        .ok_or_else(|| serde::de::Error::custom("Datime<Utc> from timestamp failed"))
}

#[derive(Debug, Clone)]
pub struct Fronter {
    pub fronter_id: String,
    pub name: String,
    pub avatar_url: String,
    pub vrchat_status_name: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub privacy_buckets: Vec<String>,
}

impl Fronter {
    #[must_use]
    pub fn preferred_vrchat_status_name(&self) -> &str {
        self.vrchat_status_name.as_ref().unwrap_or(&self.name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomFront {
    pub content: CustomFrontContent,
    #[serde(rename = "id")]
    pub custom_front_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomFrontContent {
    pub name: String,

    #[serde(rename = "avatarUrl")]
    #[serde(default)]
    pub avatar_url: String,

    #[serde(rename = "buckets")]
    pub privacy_buckets: Vec<String>,
}

impl From<CustomFront> for Fronter {
    fn from(cf: CustomFront) -> Self {
        Self {
            fronter_id: cf.custom_front_id,
            name: cf.content.name,
            avatar_url: cf.content.avatar_url,
            vrchat_status_name: None,
            start_time: None,
            privacy_buckets: cf.content.privacy_buckets,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Member {
    pub content: MemberContent,
    #[serde(rename = "id")]
    pub member_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MemberContent {
    pub name: String,

    #[serde(rename = "avatarUrl")]
    #[serde(default)]
    pub avatar_url: String,

    #[serde(default)]
    pub info: serde_json::Value,
    // if the user uses the custom field "VRChat Status Name" on this member, then this will be
    // { "<vrcsn_field_id>": "<vrcsn>", ...}
    pub archived: bool,

    #[serde(rename = "preventsFrontNotifs")]
    pub front_notifications_disabled: bool,

    /* the fields `private` and `preventTrusted` are always true for all members (according to our testing)!
    so it doesn't mean what we mean by member privacy.
    hence, we don't include it in our implementation
    */
    #[serde(rename = "buckets")]
    pub privacy_buckets: Vec<String>,

    // this will be populated later after deserialisation
    #[serde(default)]
    pub vrcsn_field_id: Option<String>,
}

impl From<Member> for Fronter {
    fn from(m: Member) -> Self {
        let vrchat_status_name = m.content.vrcsn_field_id.as_ref().and_then(|field_id| {
            m.content
                .info
                .as_object()
                .and_then(|custom_fields| custom_fields.get(field_id))
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        });
        Self {
            fronter_id: m.member_id,
            name: m.content.name,
            avatar_url: m.content.avatar_url,
            vrchat_status_name,
            start_time: None,
            privacy_buckets: m.content.privacy_buckets,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomField {
    pub id: String, // custom field id
    pub content: CustomFieldContent,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomFieldContent {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Friend {
    pub content: FriendContent,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FriendContent {
    #[serde(rename = "frienduid")]
    pub friend_user_id: String,

    #[serde(rename = "buckets")]
    pub assigned_privacy_buckets: Vec<String>,
}
