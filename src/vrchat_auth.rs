use crate::config::UserConfig;

use anyhow::{anyhow, Error, Result};
use reqwest::cookie::{self, CookieStore};
use reqwest::Url;
use std::io::{self, Write};
use std::str::FromStr;
use std::sync::Arc;
use vrchatapi::{
    apis::{authentication_api, configuration::Configuration},
    models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode},
};

const VRCHAT_UPDATER_USER_AGENT: &str = concat!(
    "SimplyPlural2Any/",
    env!("CARGO_PKG_VERSION"),
    " golly.ticker",
    "@",
    "gmail.com"
);

const VRCHAT_COOKIE_URL: &str = "https://api.vrchat.cloud";

pub async fn authenticate_vrchat(config: &UserConfig) -> Result<(Configuration, String)> {
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
    config: &UserConfig,
    cookie_store: &Arc<cookie::Jar>,
) -> Result<Configuration> {
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;

    cookie_store.add_cookie_str(&config.vrchat_cookie, cookie_url);

    let vrchat_config = Configuration {
        user_agent: Some(VRCHAT_UPDATER_USER_AGENT.to_string()),
        basic_auth: Some((
            config.vrchat_username.clone(),
            Some(config.vrchat_password.clone()),
        )),
        client: reqwest::Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()?,
        ..Default::default()
    };

    Ok(vrchat_config)
}

async fn authenticate_vrchat_user(
    config: &UserConfig,
    vrchat_config: &Configuration,
) -> Result<bool, Error> {
    let new_cookie_recieved_due_to_2fa = match authentication_api::get_current_user(vrchat_config)
        .await?
    {
        EitherUserOrTwoFactor::CurrentUser(_me) => false,

        EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            // either cookie was empty or invalid. we mark the cookie as such then

            if requires_auth
                .requires_two_factor_auth
                .contains(&String::from("emailOtp"))
            {
                let code = read_user_input(&format!(
                    "Your account {} has received an Email with a 2FA code. Please enter it: ",
                    config.vrchat_username
                ))?;
                authentication_api::verify2_fa_email_code(
                    vrchat_config,
                    TwoFactorEmailCode::new(code),
                )
                .await?;
            } else {
                let code = read_user_input(&format!(
                    "Please enter your Authenticator 2FA code for the account {}:",
                    config.vrchat_username
                ))?;
                authentication_api::verify2_fa(vrchat_config, TwoFactorAuthCode::new(code)).await?;
            }

            true
        }
    };

    Ok(new_cookie_recieved_due_to_2fa)
}

fn store_new_cookie(config: &UserConfig, cookie_store: &Arc<cookie::Jar>) -> Result<()> {
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;
    let cookie_value = cookie_store
        .cookies(cookie_url)
        .ok_or_else(|| Error::msg("no cookies"))?;
    todo!() // store_vrchat_cookie( cookie_value.to_str()? )
}

async fn get_vrchat_user_id(config: &UserConfig, vrchat_config: &Configuration) -> Result<String> {
    match authentication_api::get_current_user(vrchat_config).await? {
        EitherUserOrTwoFactor::CurrentUser(user) => Ok(user.id),
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!(
            "Cookie invalid for user {}",
            config.vrchat_username
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
