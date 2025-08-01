use std::string::ToString;

use anyhow::Result;
use serde::Deserialize;

use crate::config::Config;

pub async fn fetch_fronts(config: &Config) -> Result<Vec<MemberContent>> {
    let front_entries = simply_plural_http_request_get_fronters(config).await?;

    if front_entries.is_empty() {
        return Ok(vec![]);
    }

    let system_id = &front_entries[0].content.uid.clone();

    let fronts = enrich_fronter_ids_with_member_info(front_entries, system_id, config).await?;

    let vrcsn_field_id = get_vrchat_status_name_field_id(config, system_id).await?;

    let fronts_with_vrchat_custom_field =
        enrich_fronters_with_vrchat_status_field(&fronts, &vrcsn_field_id);

    Ok(fronts_with_vrchat_custom_field)
}

#[allow(clippy::ref_option)]
fn enrich_fronters_with_vrchat_status_field(
    fronts: &[MemberContent],
    vrcsn_field_id: &Option<String>,
) -> Vec<MemberContent> {
    fronts
        .iter()
        .map(|m| {
            let mut enriched_member = m.clone();
            enriched_member.vrcsn_field_id.clone_from(vrcsn_field_id);
            println!("Fronting member: {enriched_member:?}");
            enriched_member
        })
        .collect()
}

async fn enrich_fronter_ids_with_member_info(
    front_entries: Vec<FrontEntry>,
    system_id: &String,
    config: &Config,
) -> Result<Vec<MemberContent>> {
    let all_members = simply_plural_http_get_members(config, system_id).await?;

    let all_custom_fronts = simply_plural_http_get_custom_fronts(config, system_id).await?;

    eprintln!("custom fronts: {all_custom_fronts:?}");
    // todo. continue here
    // todo. return both kinds of results
    // todo. add test where custom front is used alone as well as with others

    let fronters: Vec<String> = front_entries
        .iter()
        .map(|e| e.content.member.clone())
        .collect();

    let enriched_fronting_members: Vec<MemberContent> = all_members
        .into_iter()
        .filter(|m| fronters.contains(&m.id))
        .map(|m| m.content)
        .collect();

    Ok(enriched_fronting_members)
}

async fn simply_plural_http_request_get_fronters(config: &Config) -> Result<Vec<FrontEntry>> {
    eprintln!("Fetching fronts from SimplyPlural...");
    let fronts_url = format!("{}/fronters", &config.simply_plural_base_url);
    let result = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.simply_plural_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(result)
}

async fn get_vrchat_status_name_field_id(
    config: &Config,
    system_id: &String,
) -> Result<Option<String>> {
    eprintln!("Fetching custom fields from SimplyPlural...");
    let custom_fields_url = format!(
        "{}/customFields/{}",
        &config.simply_plural_base_url, system_id
    );
    let custom_fields: Vec<CustomField> = config
        .client
        .get(&custom_fields_url)
        .header("Authorization", &config.simply_plural_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let vrchat_status_name_field = custom_fields
        .iter()
        .find(|field| field.content.name == "VRChat Status Name");

    let field_id = vrchat_status_name_field.map(|field| &field.id);

    Ok(field_id.cloned())
}

async fn simply_plural_http_get_members(
    config: &Config,
    system_id: &String,
) -> Result<Vec<Member>> {
    eprintln!("Fetching all members from SimplyPlural..");
    let fronts_url = format!("{}/members/{}", &config.simply_plural_base_url, system_id);
    let result = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.simply_plural_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(result)
}

async fn simply_plural_http_get_custom_fronts(
    config: &Config,
    system_id: &String,
) -> Result<Vec<CustomFront>> {
    eprintln!("Fetching all Custom Fronts from SimplyPlural...");
    let custom_fronts_url = format!(
        "{}/customFronts/{}",
        &config.simply_plural_base_url, system_id
    );
    let result = config
        .client
        .get(&custom_fronts_url)
        .header("Authorization", &config.simply_plural_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(result)
}

#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntry {
    pub content: FrontEntryContent,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FrontEntryContent {
    pub member: String, // member ID or custom front ID
    pub uid: String,    // System ID
}

/*
[
    {
        "exists": true,
        "id": "688d41c8aa2e477e53000000",
        "content": {
            "name": "Cstm First",
            "desc": "",
            "avatarUrl": "",
            "preventTrusted": true,
            "private": true,
            "supportDescMarkdown": true,
            "preventsFrontNotifs": false,
            "color": "#aa5b43",
            "uid": "8cb797f91533438f8507bcebf14febd18dc605852521d117f21358f6d950480c",
            "lastOperationTime": 1754087910796,
            "buckets": [],
            "avatarUuid": "",
            "frame": {
                "bgShape": "empty",
                "bgClip": "none",
                "bgStartColor": "#aa5b43",
                "bgEndColor": "#aa5b43"
            }
        }
    }
]
*/

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

impl MemberContent {
    pub fn preferred_vrchat_status_name(&self) -> String {
        self.vrcsn_field_id.as_ref().map_or_else(
            || self.name.clone(),
            |field_id| {
                self.info
                    .as_object()
                    .and_then(|custom_fields| custom_fields.get(field_id))
                    .and_then(|value| value.as_str())
                    .map_or_else(|| self.name.clone(), ToString::to_string)
            },
        )
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
