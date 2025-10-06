use std::collections::HashSet;

use anyhow::{Result, anyhow};

use crate::{
    int_counter_metric, int_gauge_metric,
    plurality::{
        CustomField, CustomFront, Friend, FrontEntry, Fronter,
        GLOBAL_SP2ANY_ON_SIMPLY_PLURAL_USER_ID, Member,
        SIMPLY_PLURAL_VRCHAT_STATUS_NAME_FIELD_NAME,
    },
    users::{self, PrivacyFineGrained},
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

    let system_id = &front_entries[0].content.system_id.clone();

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
    system_id: &str,
    vrcsn_field_id: Option<String>,
    config: &users::UserConfigForUpdater,
) -> Result<Vec<Fronter>> {
    let all_members: Vec<Member> = simply_plural_http_get_members(config, system_id)
        .await?
        .iter()
        .filter(|m| show_member_according_to_privacy_rules(config, m))
        .cloned()
        .collect();

    let all_custom_fronts: Vec<CustomFront> = if config.show_custom_fronts {
        let custom_fronts = simply_plural_http_get_custom_fronts(config, system_id).await?;

        SIMPLY_PLURAL_FETCH_FRONTS_CUSTOM_FRONTS_COUNT
            .with_label_values(&[&config.user_id.to_string()])
            .set(custom_fronts.len() as i64);

        custom_fronts
    } else {
        vec![]
    };

    SIMPLY_PLURAL_FETCH_FRONTS_MEMBERS_COUNT
        .with_label_values(&[&config.user_id.to_string()])
        .set(all_members.len() as i64);

    let all_frontables: Vec<Fronter> = all_members
        .into_iter()
        .map(|m| {
            let mut enriched_member = m;
            enriched_member
                .content
                .vrcsn_field_id
                .clone_from(&vrcsn_field_id);
            enriched_member
        })
        .map(Fronter::from)
        .chain(all_custom_fronts.into_iter().map(Fronter::from))
        .collect();

    let fine_grained_filtered_frontables =
        filter_frontables_by_fine_grained_privacy(system_id, config, all_frontables).await?;

    Ok(fine_grained_filtered_frontables)
}

async fn filter_frontables_by_fine_grained_privacy(
    system_id: &str,
    config: &users::UserConfigForUpdater,
    all_frontables: Vec<Fronter>,
) -> Result<Vec<Fronter>> {
    let allowed_buckets = match config.privacy_fine_grained {
        PrivacyFineGrained::NoFineGrained => return Ok(all_frontables),
        PrivacyFineGrained::ViaFriend => {
            simply_plural_http_request_get_sp2any_assigned_buckets(config, system_id).await?
        }
        PrivacyFineGrained::ViaPrivacyBuckets => config
            .privacy_fine_grained_buckets
            .as_ref()
            .ok_or_else(|| anyhow!("privacy_fine_grained_buckets must be set"))?
            .iter()
            .cloned()
            .collect(),
    };

    let privacy_bucket_filtered = all_frontables
        .into_iter()
        .filter(|f| {
            f.privacy_buckets
                .iter()
                .any(|b| allowed_buckets.contains(b))
        })
        .collect();

    Ok(privacy_bucket_filtered)
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
                .find(|fe| fe.content.fronter_id == f.fronter_id)
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
        .find(|field| field.content.name == SIMPLY_PLURAL_VRCHAT_STATUS_NAME_FIELD_NAME);

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
    system_id: &str,
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
    system_id: &str,
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

async fn simply_plural_http_request_get_sp2any_assigned_buckets(
    config: &users::UserConfigForUpdater,
    system_id: &str,
) -> Result<HashSet<String>> {
    log::info!(
        "# | simply_plural_http_request_get_sp2any_assigned_buckets | {}",
        config.user_id
    );
    let friend_url = format!(
        "{}/friend/{}/{}",
        &config.simply_plural_base_url, system_id, GLOBAL_SP2ANY_ON_SIMPLY_PLURAL_USER_ID
    );
    let friend: Friend = config
        .client
        .get(&friend_url)
        .header("Authorization", &config.simply_plural_token.secret)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let allowed_buckets = friend
        .content
        .assigned_privacy_buckets
        .into_iter()
        .collect();

    Ok(allowed_buckets)
}
