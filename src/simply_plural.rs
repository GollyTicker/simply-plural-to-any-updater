use anyhow::Result;
use serde::Deserialize;
use serde_json;

use crate::config::Config;

pub async fn fetch_fronts(config: &Config) -> Result<Vec<Fronter>> {
    let front_entries = simply_plural_http_request_get_fronters(config).await?;

    if front_entries.is_empty() {
        return Ok(vec![]);
    }

    let system_id = &front_entries[0].content.uid.clone();

    let vrcsn_field_id = get_vrchat_status_name_field_id(config, system_id).await?;

    let frontables = get_all_members_and_custom_fronters(system_id, vrcsn_field_id, config).await?;

    let fronters = filter_frontables_by_front_entries(front_entries, frontables);

    for f in &fronters {
        eprintln!("Fronter: {f:?}");
    }

    Ok(fronters)
}

async fn get_all_members_and_custom_fronters(
    system_id: &String,
    vrcsn_field_id: Option<String>,
    config: &Config,
) -> Result<Vec<Fronter>> {
    let all_members: Vec<Fronter> = simply_plural_http_get_members(config, system_id)
        .await?
        .iter()
        .map(|m| {
            let mut enriched_member = m.clone();
            enriched_member
                .content
                .vrcsn_field_id
                .clone_from(&vrcsn_field_id);
            enriched_member
        })
        .map(|m| fronter_from_member(&m))
        .collect();

    let all_custom_fronts: Vec<Fronter> = simply_plural_http_get_custom_fronts(config, system_id)
        .await?
        .iter()
        .map(fronter_from_custom_front)
        .collect();

    let all_frontables: Vec<Fronter> =
        [all_members.as_slice(), all_custom_fronts.as_slice()].concat();

    Ok(all_frontables)
}

#[allow(clippy::needless_pass_by_value)]
fn filter_frontables_by_front_entries(
    front_entries: Vec<FrontEntry>,
    frontables: Vec<Fronter>,
) -> Vec<Fronter> {
    let fronter_ids: Vec<String> = front_entries
        .iter()
        .map(|e| e.content.member.clone())
        .collect();

    let fronters: Vec<Fronter> = frontables
        .into_iter()
        .filter(|f| fronter_ids.contains(&f.id))
        .collect();

    fronters
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

#[derive(Debug, Clone)]
pub struct Fronter {
    pub(crate) id: String,
    pub name: String,
    pub avatar_url: String,
    pub(crate) vrchat_status_name: Option<String>,
}

impl Fronter {
    pub fn preferred_vrchat_status_name(&self) -> String {
        self.vrchat_status_name
            .clone()
            .unwrap_or_else(|| self.name.clone())
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

fn fronter_from_custom_front(cf: &CustomFront) -> Fronter {
    Fronter {
        id: cf.id.clone(),
        name: cf.content.name.clone(),
        avatar_url: cf.content.avatar_url.clone(),
        vrchat_status_name: None,
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

fn fronter_from_member(m: &Member) -> Fronter {
    let vrchat_status_name = m.content.vrcsn_field_id.as_ref().and_then(|field_id| {
        m.content
            .info
            .as_object()
            .and_then(|custom_fields| custom_fields.get(field_id))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
    });
    Fronter {
        id: m.id.clone(),
        name: m.content.name.clone(),
        avatar_url: m.content.avatar_url.clone(),
        vrchat_status_name,
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
