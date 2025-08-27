use anyhow::Result;

use crate::{
    plurality::{CustomField, CustomFront, FrontEntry, Fronter, Member},
    users,
};

pub async fn fetch_fronts(config: &users::UserConfigForUpdater) -> Result<Vec<Fronter>> {
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
    config: &users::UserConfigForUpdater,
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
        .map(Fronter::from)
        .collect();

    let all_custom_fronts: Vec<Fronter> = simply_plural_http_get_custom_fronts(config, system_id)
        .await?
        .iter()
        .cloned()
        .map(Fronter::from)
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

async fn simply_plural_http_request_get_fronters(
    config: &users::UserConfigForUpdater,
) -> Result<Vec<FrontEntry>> {
    eprintln!("Fetching fronts from SimplyPlural...");
    let fronts_url = format!("{}/fronters", &config.simply_plural_base_url);
    let result = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.simply_plural_token.secret)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(result)
}

async fn get_vrchat_status_name_field_id(
    config: &users::UserConfigForUpdater,
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
        .header("Authorization", &config.simply_plural_token.secret)
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
    config: &users::UserConfigForUpdater,
    system_id: &String,
) -> Result<Vec<Member>> {
    eprintln!("Fetching all members from SimplyPlural..");
    let fronts_url = format!("{}/members/{}", &config.simply_plural_base_url, system_id);
    let result = config
        .client
        .get(&fronts_url)
        .header("Authorization", &config.simply_plural_token.secret)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(result)
}

async fn simply_plural_http_get_custom_fronts(
    config: &users::UserConfigForUpdater,
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
        .header("Authorization", &config.simply_plural_token.secret)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(result)
}
