use std::sync::Arc;

use crate::users;

use serde::{Deserialize, Serialize};
use strum_macros::Display;
use vrchatapi::models::current_user::RequiresTwoFactorAuth;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct VRChatUserId {
    pub inner: String,
}

pub type Cookies = Arc<reqwest_cookie_store::CookieStoreMutex>;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct VRChatCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct VRChatCredentialsWithCookie {
    pub creds: VRChatCredentials,
    pub cookie: String,
}

impl VRChatCredentialsWithCookie {
    pub fn from_config(config: &users::UserConfigForUpdater) -> Self {
        Self::from_strings(
            config.vrchat_username.secret.clone(),
            config.vrchat_password.secret.clone(),
            config.vrchat_cookie.secret.clone(),
        )
    }

    pub fn from(creds: VRChatCredentials, cookie: String) -> Self {
        Self::from_strings(creds.username, creds.password, cookie)
    }

    const fn from_strings(username: String, password: String, cookie: String) -> Self {
        Self {
            creds: VRChatCredentials { username, password },
            cookie,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorCodeRequiredResponse {
    pub method: TwoFactorAuthMethod,
    pub tmp_cookie: String,
}

#[derive(Clone, Serialize, Deserialize, Display, Debug)]
pub enum TwoFactorAuthMethod {
    TwoFactorAuthMethodEmail,
    TwoFactorAuthMethodApp,
}

impl TwoFactorAuthMethod {
    pub fn from(requires_2fa_auth: &RequiresTwoFactorAuth) -> Self {
        let is_email_2fa = requires_2fa_auth
            .requires_two_factor_auth
            .contains(&String::from("emailOtp"));

        if is_email_2fa {
            Self::TwoFactorAuthMethodEmail
        } else {
            Self::TwoFactorAuthMethodApp
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TwoFactorAuthCode {
    inner: String,
}

impl From<TwoFactorAuthCode> for String {
    fn from(val: TwoFactorAuthCode) -> Self {
        val.inner
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VRChatCredentialsWithTwoFactorAuth {
    pub creds: VRChatCredentials,
    pub method: TwoFactorAuthMethod,
    pub code: TwoFactorAuthCode,
    pub tmp_cookie: String,
}
