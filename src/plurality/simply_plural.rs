use anyhow::Result;

use crate::{
    int_counter_metric, int_gauge_metric,
    plurality::{CustomField, CustomFront, FrontEntry, Fronter, Member},
    users,
};

int_counter_metric!(SIMPLY_PLURAL_FETCH_FRONTS_TOTAL_COUNTER);
int_gauge_metric!(SIMPLY_PLURAL_FETCH_FRONTS_FRONTERS_COUNT);
int_gauge_metric!(SIMPLY_PLURAL_FETCH_FRONTS_MEMBERS_COUNT);
int_gauge_metric!(SIMPLY_PLURAL_FETCH_FRONTS_CUSTOM_FRONTS_COUNT);

#[allow(clippy::cast_possible_wrap)]
pub async fn fetch_fronts(config: &users::UserConfigForUpdater) -> Result<Vec<Fronter>> {
    let user_id = &config.user_id;

    SIMPLY_PLURAL_FETCH_FRONTS_TOTAL_COUNTER
        .with_label_values(&[&user_id.to_string()])
        .inc();

    let front_entries = simply_plural_http_request_get_fronters(config).await?;

    if front_entries.is_empty() {
        SIMPLY_PLURAL_FETCH_FRONTS_FRONTERS_COUNT
            .with_label_values(&[&user_id.to_string()])
            .set(0);
        return Ok(vec![]);
    }

    let system_id = &front_entries[0].content.uid.clone();

    let vrcsn_field_id = get_vrchat_status_name_field_id(config, system_id).await?;

    let frontables =
        get_members_and_custom_fronters_by_privacy_rules(system_id, vrcsn_field_id, config).await?;

    let fronters = filter_frontables_by_front_entries(front_entries, frontables);

    for f in &fronters {
        log::info!("# | fetch_fronts | fronter[*] {f:?}");
    }

    SIMPLY_PLURAL_FETCH_FRONTS_FRONTERS_COUNT
        .with_label_values(&[&user_id.to_string()])
        .set(fronters.len() as i64);

    Ok(fronters)
}

const fn show_member_according_to_privacy_rules(
    config: &users::UserConfigForUpdater,
    member_with_content: &Member,
) -> bool {
    let member: &super::MemberContent = &member_with_content.content;

    if config.respect_front_notifications_disabled && member.front_notifications_disabled {
        return false;
    }
    if member.archived {
        return config.show_members_archived;
    }

    config.show_members_non_archived
}

#[allow(clippy::cast_possible_wrap)]
async fn get_members_and_custom_fronters_by_privacy_rules(
    system_id: &String,
    vrcsn_field_id: Option<String>,
    config: &users::UserConfigForUpdater,
) -> Result<Vec<Fronter>> {
    let all_members: Vec<Fronter> = simply_plural_http_get_members(config, system_id)
        .await?
        .iter()
        .filter(|m| show_member_according_to_privacy_rules(config, m))
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

    SIMPLY_PLURAL_FETCH_FRONTS_MEMBERS_COUNT
        .with_label_values(&[&config.user_id.to_string()])
        .set(all_members.len() as i64);

    let all_custom_fronts: Vec<Fronter> = if config.show_custom_fronts {
        simply_plural_http_get_custom_fronts(config, system_id)
            .await?
            .iter()
            .cloned()
            .map(Fronter::from)
            .collect()
    } else {
        vec![]
    };

    SIMPLY_PLURAL_FETCH_FRONTS_CUSTOM_FRONTS_COUNT
        .with_label_values(&[&config.user_id.to_string()])
        .set(all_custom_fronts.len() as i64);

    let all_frontables: Vec<Fronter> =
        [all_members.as_slice(), all_custom_fronts.as_slice()].concat();

    Ok(all_frontables)
}

#[allow(clippy::needless_pass_by_value)]
fn filter_frontables_by_front_entries(
    front_entries: Vec<FrontEntry>,
    frontables: Vec<Fronter>,
) -> Vec<Fronter> {
    let fronters: Vec<Fronter> = frontables
        .iter()
        .filter_map(|f| {
            front_entries
                .iter()
                .find(|fe| fe.content.member == f.id)
                .map(|fe| {
                    let mut fronter_with_start_time = f.clone();
                    fronter_with_start_time.start_time = Some(fe.content.start_time);
                    fronter_with_start_time
                })
        })
        .collect();

    fronters
}

async fn simply_plural_http_request_get_fronters(
    config: &users::UserConfigForUpdater,
) -> Result<Vec<FrontEntry>> {
    log::info!(
        "# | simply_plural_http_request_get_fronters | {}",
        config.user_id
    );

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
    log::info!("# | get_vrchat_status_name_field_id | {}", config.user_id);
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

    log::info!(
        "# | get_vrchat_status_name_field_id | {} | field_id {:?}",
        config.user_id,
        field_id
    );

    Ok(field_id.cloned())
}

async fn simply_plural_http_get_members(
    config: &users::UserConfigForUpdater,
    system_id: &String,
) -> Result<Vec<Member>> {
    log::info!("# | simply_plural_http_get_members | {}", config.user_id);
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
    log::info!(
        "# | simply_plural_http_get_custom_fronts | {}",
        config.user_id
    );
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
