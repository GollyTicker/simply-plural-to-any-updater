use crate::{database::secrets, users::UserConfigDbEntries};

pub trait ConstraintsType: Clone {}

/// Constraints of the configs from the DB are only valid when
/// * they're validated via config.rs and ONLY THEN put into the DB
/// * read from the DB (since they're valid before putting them in)
#[derive(Clone)]
pub struct ValidConstraints {}

#[derive(Clone, Default)]
pub struct InvalidConstraints {}

impl ConstraintsType for ValidConstraints {}
impl ConstraintsType for InvalidConstraints {}

impl From<Option<bool>> for InvalidConstraints {
    fn from(_: Option<bool>) -> Self {
        Self {}
    }
}

impl From<Option<bool>> for ValidConstraints {
    fn from(_: Option<bool>) -> Self {
        Self {}
    }
}

pub fn downgrade<Secret: secrets::SecretType, C: ConstraintsType>(
    value: &UserConfigDbEntries<Secret, C>,
) -> UserConfigDbEntries<Secret, InvalidConstraints> {
    UserConfigDbEntries {
        valid_constraints: InvalidConstraints {},
        wait_seconds: value.wait_seconds,
        system_name: value.system_name.clone(),
        status_prefix: value.status_prefix.clone(),
        status_no_fronts: value.status_no_fronts.clone(),
        status_truncate_names_to: value.status_truncate_names_to,
        enable_discord: value.enable_discord,
        enable_discord_status_message: value.enable_discord_status_message,
        enable_vrchat: value.enable_vrchat,
        simply_plural_token: value.simply_plural_token.clone(),
        discord_status_message_token: value.discord_status_message_token.clone(),
        discord_user_id: value.discord_user_id.clone(),
        discord_oauth_access_token: value.discord_oauth_access_token.clone(),
        discord_oauth_refresh_token: value.discord_oauth_refresh_token.clone(),
        vrchat_username: value.vrchat_username.clone(),
        vrchat_password: value.vrchat_password.clone(),
        vrchat_cookie: value.vrchat_cookie.clone(),
    }
}

pub fn only_use_this_function_to_mark_validation_after_you_have_actually_validated_it<
    Secret: secrets::SecretType,
>(
    value: &UserConfigDbEntries<Secret, InvalidConstraints>,
) -> UserConfigDbEntries<Secret, ValidConstraints> {
    UserConfigDbEntries {
        valid_constraints: ValidConstraints {},
        wait_seconds: value.wait_seconds,
        system_name: value.system_name.clone(),
        status_prefix: value.status_prefix.clone(),
        status_no_fronts: value.status_no_fronts.clone(),
        status_truncate_names_to: value.status_truncate_names_to,
        enable_discord: value.enable_discord,
        enable_discord_status_message: value.enable_discord_status_message,
        enable_vrchat: value.enable_vrchat,
        simply_plural_token: value.simply_plural_token.clone(),
        discord_status_message_token: value.discord_status_message_token.clone(),
        discord_user_id: value.discord_user_id.clone(),
        discord_oauth_access_token: value.discord_oauth_access_token.clone(),
        discord_oauth_refresh_token: value.discord_oauth_refresh_token.clone(),
        vrchat_username: value.vrchat_username.clone(),
        vrchat_password: value.vrchat_password.clone(),
        vrchat_cookie: value.vrchat_cookie.clone(),
    }
}
