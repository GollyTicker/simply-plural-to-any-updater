use crate::config::UserConfigForUpdater;

use anyhow::{anyhow, Result};
use reqwest::cookie::{self, CookieStore};
use reqwest::Url;
use std::io::{self, Write};
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

pub async fn authenticate_vrchat(config: &UserConfigForUpdater) -> Result<(VrcConfig, String)> {
    let cookie_store = Arc::new(cookie::Jar::default());

    let vrchat_config =
        new_vrchat_config_with_basic_auth_and_optional_cookie(config, &cookie_store)?;

    let new_cookie_received = authenticate_vrchat_user(config, &vrchat_config).await?;

    if new_cookie_received {
        store_new_cookie(config, &cookie_store)?;
    }

    let user_id = get_vrchat_user_id(config, &vrchat_config).await?;

    Ok((vrchat_config, user_id))
}

fn new_vrchat_config_with_basic_auth_and_optional_cookie(
    config: &UserConfigForUpdater,
    cookie_store: &Arc<cookie::Jar>,
) -> Result<VrcConfig> {
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;

    cookie_store.add_cookie_str(&config.vrchat_cookie.secret, cookie_url);

    let vrchat_config = VrcConfig {
        user_agent: Some(VRCHAT_UPDATER_USER_AGENT.to_string()),
        basic_auth: Some((
            config.vrchat_username.secret.clone(),
            Some(config.vrchat_password.secret.clone()),
        )),
        client: reqwest::Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()?,
        ..Default::default()
    };

    Ok(vrchat_config)
}

async fn authenticate_vrchat_user(
    config: &UserConfigForUpdater,
    vrchat_config: &VrcConfig,
) -> Result<bool> {
    let new_cookie_recieved_due_to_2fa = match authentication_api::get_current_user(vrchat_config)
        .await?
    {
        vrc::EitherUserOrTwoFactor::CurrentUser(_me) => false,

        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            // either cookie was empty or invalid. we mark the cookie as such then

            if requires_auth
                .requires_two_factor_auth
                .contains(&String::from("emailOtp"))
            {
                let code = read_user_input(&format!(
                    "Your account {} has received an Email with a 2FA code. Please enter it: ",
                    config.vrchat_username.secret
                ))?;
                authentication_api::verify2_fa_email_code(
                    vrchat_config,
                    vrc::TwoFactorEmailCode::new(code),
                )
                .await?;
            } else {
                let code = read_user_input(&format!(
                    "Please enter your Authenticator 2FA code for the account {}:",
                    config.vrchat_username.secret
                ))?;
                authentication_api::verify2_fa(vrchat_config, vrc::TwoFactorAuthCode::new(code))
                    .await?;
            }

            true
        }
    };

    Ok(new_cookie_recieved_due_to_2fa)
}

fn store_new_cookie(config: &UserConfigForUpdater, cookie_store: &Arc<cookie::Jar>) -> Result<()> {
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;
    let cookie_value = cookie_store
        .cookies(cookie_url)
        .ok_or_else(|| anyhow!("no cookies"))?;
    todo!() // store_vrchat_cookie( cookie_value.to_str()? )
}

async fn get_vrchat_user_id(
    config: &UserConfigForUpdater,
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

fn read_user_input(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
