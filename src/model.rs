use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sqlx::{
    error::BoxDynError, postgres::PgValueRef, types::Uuid, Decode, FromRow, Postgres, Type,
};

use crate::config::UserConfigDbEntries;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, Type)]
pub struct Email {
    pub inner: String,
}

impl From<String> for Email {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, Type, Eq, Hash, PartialEq)]
pub struct UserId {
    pub inner: Uuid,
}

impl From<Uuid> for UserId {
    fn from(val: Uuid) -> Self {
        Self { inner: val }
    }
}

impl TryFrom<&str> for UserId {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let uuid = Uuid::from_str(value)?;
        Ok(Self { inner: uuid })
    }
}

#[derive(Deserialize, Clone)]
pub struct UserProvidedPassword {
    pub inner: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Type)]
pub struct PasswordHashString {
    pub inner: String,
}
impl From<String> for PasswordHashString {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

#[derive(Clone)]
pub struct UserSecretsKey {
    pub inner: String,
}

pub struct ApplicationJwtSecret {
    pub inner: String,
}

pub struct ApplicationUserSecrets {
    pub inner: String,
}

#[derive(Deserialize, Clone)]
pub struct UserLoginCredentials {
    pub email: Email,
    pub password: UserProvidedPassword,
}

#[derive(Default, Clone, Serialize, FromRow, PartialEq, Eq)]
pub struct EncryptedDbSecret {}

impl From<String> for EncryptedDbSecret {
    fn from(_: String) -> Self {
        Self {}
    }
}

// manual implementation of `Type<Postgres>` because derive doesn't work for non-newtype structs
impl Type<Postgres> for EncryptedDbSecret {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for EncryptedDbSecret {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let _ = <String as Decode<Postgres>>::decode(value)?;
        Ok(Self {})
    }
}

#[derive(Default, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct DecryptedDbSecret {
    pub secret: String,
}

// manual implementation of `Type<Postgres>` because derive doesn't work for non-newtype structs
impl Type<Postgres> for DecryptedDbSecret {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for DecryptedDbSecret {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let secret = <String as Decode<Postgres>>::decode(value)?;
        Ok(Self { secret })
    }
}

impl From<&str> for DecryptedDbSecret {
    fn from(value: &str) -> Self {
        Self {
            secret: value.to_string(),
        }
    }
}

impl From<String> for DecryptedDbSecret {
    fn from(secret: String) -> Self {
        Self { secret }
    }
}

pub trait SecretType: Default + Clone {}
impl SecretType for EncryptedDbSecret {}
impl SecretType for DecryptedDbSecret {}

pub trait ConstraintsType: Clone {}

/// Constraints of the configs from the DB are only valid when
/// * they're validated via config.rs and ONLY THEN put into the DB
/// * read from the DB (since they're valid before putting them in)
#[derive(Clone)]
pub struct ValidConstraints {}

#[derive(Clone, Default, FromRow)]
pub struct InvalidConstraints {}

impl ConstraintsType for ValidConstraints {}
impl ConstraintsType for InvalidConstraints {}

// manual implementation of `Type<Postgres>` because derive doesn't work for non-newtype structs
impl Type<Postgres> for ValidConstraints {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <bool as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <bool as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for ValidConstraints {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let valid_constraints = <bool as Decode<Postgres>>::decode(value)?;

        if valid_constraints {
            Ok(Self {})
        } else {
            Err(anyhow!("Implementation bug! (49273)").into())
        }
    }
}

// manual implementation of `Type<Postgres>` because derive doesn't work for non-newtype structs
impl Type<Postgres> for InvalidConstraints {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <bool as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <bool as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for InvalidConstraints {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let valid_constraints = <bool as Decode<Postgres>>::decode(value)?;

        if valid_constraints {
            Err(anyhow!("Implementation bug! (49273)").into())
        } else {
            Ok(Self {})
        }
    }
}

pub fn downgrade<Secret: SecretType, C: ConstraintsType>(
    value: &UserConfigDbEntries<Secret, C>,
) -> UserConfigDbEntries<Secret, InvalidConstraints> {
    UserConfigDbEntries {
        valid_constraints: Some(InvalidConstraints {}),
        wait_seconds: value.wait_seconds,
        system_name: value.system_name.clone(),
        status_prefix: value.status_prefix.clone(),
        status_no_fronts: value.status_no_fronts.clone(),
        status_truncate_names_to: value.status_truncate_names_to,
        enable_discord: value.enable_discord,
        enable_vrchat: value.enable_vrchat,
        simply_plural_token: value.simply_plural_token.clone(),
        discord_token: value.discord_token.clone(),
        vrchat_username: value.vrchat_username.clone(),
        vrchat_password: value.vrchat_password.clone(),
        vrchat_cookie: value.vrchat_cookie.clone(),
    }
}

pub fn only_use_this_function_to_mark_validation_after_you_have_actually_validated_it<
    Secret: SecretType,
>(
    value: &UserConfigDbEntries<Secret, InvalidConstraints>,
) -> UserConfigDbEntries<Secret, ValidConstraints> {
    UserConfigDbEntries {
        valid_constraints: Some(ValidConstraints {}),
        wait_seconds: value.wait_seconds,
        system_name: value.system_name.clone(),
        status_prefix: value.status_prefix.clone(),
        status_no_fronts: value.status_no_fronts.clone(),
        status_truncate_names_to: value.status_truncate_names_to,
        enable_discord: value.enable_discord,
        enable_vrchat: value.enable_vrchat,
        simply_plural_token: value.simply_plural_token.clone(),
        discord_token: value.discord_token.clone(),
        vrchat_username: value.vrchat_username.clone(),
        vrchat_password: value.vrchat_password.clone(),
        vrchat_cookie: value.vrchat_cookie.clone(),
    }
}
