use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::{plurality::deserialize_non_empty_string_as_option, users};

pub const PLURALKIT_USER_AGENT: &str = concat!(
    "PluralSync/",
    env!("CARGO_PKG_VERSION"),
    " Discord: ",
    env!("USER_AGENT_DISCORD_USERNAME")
);

#[derive(Deserialize, Debug, Clone)]
pub struct PluralKitMember {
    pub id: String,
    pub name: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_non_empty_string_as_option")]
    pub display_name: Option<String>,
}

pub async fn get_pluralkit_members(
    config: &users::UserConfigForUpdater,
) -> Result<HashMap<String, PluralKitMember>> {
    let response = config
        .client
        .get("https://api.pluralkit.me/v2/systems/@me/members")
        .header("Authorization", &config.pluralkit_token.secret)
        .header("User-Agent", PLURALKIT_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let members: Vec<PluralKitMember> = serde_json::from_str(&response).inspect_err(|e| {
        log::warn!(
            "# | get_pluralkit_members | {} | {} | input: {}",
            config.user_id,
            e,
            response.chars().take(500).collect::<String>()
        );
    })?;

    let members_map = members.into_iter().map(|m| (m.id.clone(), m)).collect();

    Ok(members_map)
}
