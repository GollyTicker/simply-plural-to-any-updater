use crate::platforms::vrchat_auth_types::{
    TwoFactorAuthCode, TwoFactorAuthMethod, TwoFactorCodeRequiredResponse, VRChatCredentials,
    VRChatCredentialsWithCookie, VRChatCredentialsWithTwoFactorAuth,
};
use crate::users;

use anyhow::{anyhow, Result};
use either::Either;
use reqwest::cookie::{self, CookieStore};
use reqwest::Url;
use std::str::FromStr;
use std::sync::Arc;
use vrchatapi::{
    apis::{authentication_api, configuration::Configuration as VrcConfig},
    models as vrc,
};

const VRCHAT_UPDATER_USER_AGENT: &str = concat!(
    "SimplyPlural2Any/",
    env!("CARGO_PKG_VERSION"),
    " golly.ticker",
    "@",
    "gmail.com"
);

const VRCHAT_COOKIE_URL: &str = "https://api.vrchat.cloud";

/* Called in updater. Cookie is only validated, no new cookie is created. */
pub async fn authenticate_vrchat_with_cookie(
    config: &users::UserConfigForUpdater,
) -> Result<(VrcConfig, String)> {
    let creds = VRChatCredentialsWithCookie::from_config(config);

    let (vrchat_config, _) =
        new_vrchat_config_with_basic_auth_and_optional_cookie(Either::Right(&creds))?;

    let () = match authentication_api::get_current_user(&vrchat_config).await? {
        vrc::EitherUserOrTwoFactor::CurrentUser(_me) => {
            eprintln!("VRChat Cookie valid for {}", config.user_id);
            Ok(())
        }
        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!("Login failed")),
    }?;

    let user_id = get_vrchat_user_id(config, &vrchat_config).await?;

    Ok((vrchat_config, user_id))
}

pub async fn authenticate_vrchat_for_new_cookie(
    creds: VRChatCredentials,
) -> Result<Either<VRChatCredentialsWithCookie, TwoFactorCodeRequiredResponse>> {
    let (vrchat_config, cookie_store) =
        new_vrchat_config_with_basic_auth_and_optional_cookie(Either::Left(&creds))?;

    match authentication_api::get_current_user(&vrchat_config).await? {
        // User doesn't need 2fa
        vrc::EitherUserOrTwoFactor::CurrentUser(_me) => {
            let cookie = extract_new_cookie(&cookie_store)?;
            let creds_with_cookie = VRChatCredentialsWithCookie::from(creds, cookie);
            Ok(Either::Left(creds_with_cookie))
        }

        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            let method = TwoFactorAuthMethod::from(&requires_auth);
            let tmp_cookie = extract_new_cookie(&cookie_store)?;
            Ok(Either::Right(TwoFactorCodeRequiredResponse {
                method,
                tmp_cookie,
            }))
        }
    }
}

pub async fn authenticate_vrchat_for_new_cookie_with_2fa(
    creds_with_tfa: VRChatCredentialsWithTwoFactorAuth,
) -> Result<VRChatCredentialsWithCookie> {
    let creds_with_tmp_cookie = VRChatCredentialsWithCookie {
        creds: creds_with_tfa.creds.clone(),
        cookie: creds_with_tfa.tmp_cookie.clone(),
    };

    let (vrchat_config, cookie_store) = new_vrchat_config_with_basic_auth_and_optional_cookie(
        Either::Right(&creds_with_tmp_cookie),
    )?;

    let () = vrchat_verify_2fa(creds_with_tfa.method, creds_with_tfa.code, &vrchat_config).await?;

    let cookie = extract_new_cookie(&cookie_store)?;

    Ok(VRChatCredentialsWithCookie::from(
        creds_with_tfa.creds,
        cookie,
    ))
}

async fn vrchat_verify_2fa(
    method: TwoFactorAuthMethod,
    auth_code: TwoFactorAuthCode,
    vrchat_config: &VrcConfig,
) -> Result<()> {
    match method {
        TwoFactorAuthMethod::TwoFactorAuthMethodEmail => {
            authentication_api::verify2_fa_email_code(
                vrchat_config,
                vrc::TwoFactorEmailCode::new(auth_code.into()),
            )
            .await?;
        }
        TwoFactorAuthMethod::TwoFactorAuthMethodApp => {
            authentication_api::verify2_fa(
                vrchat_config,
                vrc::TwoFactorAuthCode::new(auth_code.into()),
            )
            .await?;
        }
    }

    Ok(())
}

fn new_vrchat_config_with_basic_auth_and_optional_cookie(
    creds: Either<&VRChatCredentials, &VRChatCredentialsWithCookie>,
) -> Result<(VrcConfig, Arc<reqwest::cookie::Jar>)> {
    let cookie_store = Arc::new(cookie::Jar::default());
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;

    let username = creds.either(|c| &c.username, |c| &c.creds.username);
    let password = creds.either(|c| &c.password, |c| &c.creds.password);
    let cookie = creds.right().map(|c| &c.cookie);

    if let Some(cookie) = cookie {
        cookie_store.add_cookie_str(cookie, cookie_url);
    }

    let vrchat_config = VrcConfig {
        user_agent: Some(VRCHAT_UPDATER_USER_AGENT.to_string()),
        basic_auth: Some((username.clone(), Some(password.clone()))),
        client: reqwest::Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()?,
        ..Default::default()
    };

    Ok((vrchat_config, cookie_store))
}

fn extract_new_cookie(cookie_store: &Arc<cookie::Jar>) -> Result<String> {
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;
    let cookie_value = cookie_store
        .cookies(cookie_url)
        .ok_or_else(|| anyhow!("no cookies"))?;
    Ok(cookie_value.to_str()?.to_owned())
}

async fn get_vrchat_user_id(
    config: &users::UserConfigForUpdater,
    vrchat_config: &VrcConfig,
) -> Result<String> {
    match authentication_api::get_current_user(vrchat_config).await? {
        vrc::EitherUserOrTwoFactor::CurrentUser(user) => Ok(user.id),
        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!(
            "Cookie invalid for user {}",
            config.vrchat_username.secret
        )),
    }
}
