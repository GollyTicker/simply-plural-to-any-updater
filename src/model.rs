use serde::{Deserialize, Serialize};
use sqlx::{
    error::BoxDynError, postgres::PgValueRef, types::Uuid, Decode, FromRow, Postgres, Row, Type,
};

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, Type)]
pub struct Email {
    pub inner: String,
}

impl From<String> for Email {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Type)]
pub struct UserId {
    pub inner: Uuid,
}

impl From<Uuid> for UserId {
    fn from(val: Uuid) -> Self {
        Self { inner: val }
    }
}

pub enum EmailOrUserId {
    Email(Email),
    UserId(UserId),
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

// manual implementation of `Type<Postgres>` because derive doesn't work :/
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

#[derive(Default, Clone, Deserialize, FromRow, Type, PartialEq, Eq)]
pub struct DecryptedDbSecret {
    pub secret: String,
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
