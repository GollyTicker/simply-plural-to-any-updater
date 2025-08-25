use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct UserSecretsDecryptionKey {
    pub inner: String,
}

#[derive(Clone)]
pub struct ApplicationUserSecrets {
    pub inner: String,
}

pub trait SecretType: Default + Clone {}
impl SecretType for Encrypted {}
impl SecretType for Decrypted {}

#[derive(Default, Clone, Serialize, PartialEq, Eq)]
pub struct Encrypted {}

impl From<String> for Encrypted {
    fn from(_: String) -> Self {
        Self {}
    }
}

impl From<Option<String>> for Encrypted {
    fn from(_: Option<String>) -> Self {
        Self {}
    }
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Decrypted {
    /// String may be empty!
    /// If the contianing config has `ValidConstraints` then this is non-empty
    /// if the conditions are met. (e.g. when `enabled_vrchat` is true, then `vrchat_username` is non-empty).
    pub secret: String,
}

impl Decrypted {
    pub fn ok_or_else<E, F>(self, err: F) -> Result<Self, E>
    where
        F: FnOnce() -> E,
    {
        let secret = self.secret;
        if secret.is_empty() {
            Err(err())
        } else {
            Ok(Self { secret })
        }
    }
}

impl From<Option<String>> for Decrypted {
    fn from(value: Option<String>) -> Self {
        Self {
            secret: value.unwrap_or_default(),
        }
    }
}

impl From<&str> for Decrypted {
    fn from(value: &str) -> Self {
        Self {
            secret: value.to_string(),
        }
    }
}

impl From<String> for Decrypted {
    fn from(secret: String) -> Self {
        Self { secret }
    }
}
