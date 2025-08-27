use std::string::ToString;

use anyhow::Result;
use serde::Deserialize;
use serde::Deserializer;

#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntry {
    pub content: FrontEntryContent,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntryContent {
    pub member: String, // member ID or custom front ID
    pub uid: String,    // System ID

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
    pub id: String,
    pub name: String,
    pub avatar_url: String,
    pub vrchat_status_name: Option<String>,
}

impl Fronter {
    pub fn preferred_vrchat_status_name(&self) -> &str {
        self.vrchat_status_name.as_ref().unwrap_or(&self.name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomFront {
    pub content: CustomFrontContent,
    pub id: String, // custom front id
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomFrontContent {
    pub name: String,

    #[serde(rename = "avatarUrl")]
    #[serde(default)]
    pub avatar_url: String,
}

impl From<CustomFront> for Fronter {
    fn from(cf: CustomFront) -> Self {
        Self {
            id: cf.id,
            name: cf.content.name,
            avatar_url: cf.content.avatar_url,
            vrchat_status_name: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Member {
    pub content: MemberContent,
    pub id: String, // member id
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
            id: m.id,
            name: m.content.name,
            avatar_url: m.content.avatar_url,
            vrchat_status_name,
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
