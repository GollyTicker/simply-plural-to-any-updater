use crate::http::HttpResult;
use crate::users::JwtString;
use crate::{database, users};
use anyhow::{anyhow, Result};
use rand::Rng;
use rand::{distr::Alphanumeric, rngs::StdRng, SeedableRng};
use reqwest::Client;
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

const DEV_OAUTH_REDIRECT_URL: &str =
    "http://localhost:8080/api/user/platform/discord/oauth/callback";

/* To test this endpoint:
1. Login into discord in browser
2. Edit the test setup, such that a user is created, but the tests sleep so that you can manually send requests.
3. Open the frontend at localhost:8080
4. Copy the JWT from the test user setup logs.
5. Paste it into the frontend and click on "Authorize Discord"
6. Authorize discord.
7. Check that authorization is successful.
*/
#[get("/api/user/platform/discord/oauth/callback?<code>&<state>")]
pub async fn get_api_auth_discord_callback(
    code: String,
    state: String, // JWT of the logged-in user
    application_discord_oauth_secrets: &State<ApplicationDiscordOAuthSecrets>,
    client: &State<reqwest::Client>,
    db_pool: &State<PgPool>,
    jwt_app_secret: &State<users::ApplicationJwtSecret>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
) -> HttpResult<RawHtml<String>> {
    // It's important that we verify the state to avoid CSRF:
    // https://discord.com/developers/docs/topics/oauth2#state-and-security
    // We have to do this anyways, as otherwise we don't know to which user this callback belongs.
    eprintln!("GET /api/user/platform/discord/oauth/callback 0");

    let sp2any_jwt_token: JwtString = serde_json::from_str(&state).map_err(|e| anyhow!(e))?;

    let user_id = users::verify_jwt(&sp2any_jwt_token, jwt_app_secret)?.user_id()?;

    eprintln!("GET /api/user/platform/discord/oauth/callback 1. verified user-id {user_id}");

    let discord_tokens =
        http_get_discord_oauth_tokens(client, &code, application_discord_oauth_secrets).await?;

    eprintln!("GET /api/user/platform/discord/oauth/callback 2. retrieved discord tokens.");

    let discord_user = http_get_discord_user_info(client, &discord_tokens.access_token).await?;

    eprintln!(
        "GET /api/user/platform/discord/oauth/callback 3. retrieved discord user info {}",
        discord_user.id
    );

    database::modify_user_secrets(
        db_pool,
        &user_id,
        client,
        application_user_secrets,
        |user_with_secrets| {
            user_with_secrets.discord_user_id = Some(discord_user.id.clone().into());
            user_with_secrets.discord_oauth_access_token =
                Some(discord_tokens.access_token.clone().into());
            user_with_secrets.discord_oauth_refresh_token =
                Some(discord_tokens.refresh_token.clone().into());
        },
    )
    .await?;

    eprintln!("GET /api/user/platform/discord/oauth/callback 4. stored secrets into database.");

    Ok(RawHtml(DISCORD_OAUTH_SUCCESS_HTML.to_owned()))
}

async fn http_get_discord_oauth_tokens(
    client: &Client,
    code: &str,
    application_discord_oauth_secrets: &ApplicationDiscordOAuthSecrets,
) -> Result<OAuthTokenResponse> {
    let token_request_body = OAuthTokenRequest {
        client_id: application_discord_oauth_secrets.client_id.clone(),
        client_secret: application_discord_oauth_secrets.client_secret.clone(),
        grant_type: "authorization_code".to_owned(),
        code: code.to_owned(),
        // This must match the redirect_uri in the initial authorization URL
        redirect_uri: DEV_OAUTH_REDIRECT_URL.to_owned(),
    };

    let token_response = client
        .post("https://discord.com/api/oauth2/token")
        .form(&token_request_body)
        .send()
        .await
        .map_err(|e| (anyhow!(e)))?
        .json::<OAuthTokenResponse>()
        .await
        .map_err(|e| (anyhow!(e)))?;

    Ok(token_response)
}

async fn http_get_discord_user_info(client: &Client, access_token: &str) -> Result<UserResponse> {
    let discord_user = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| (anyhow!(e)))?
        .json::<UserResponse>()
        .await
        .map_err(|e| (anyhow!(e)))?;

    Ok(discord_user)
}

#[derive(Clone)]
pub struct ApplicationDiscordOAuthSecrets {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Serialize)]
struct OAuthTokenRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    code: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct UserResponse {
    id: String,
}

const DISCORD_OAUTH_SUCCESS_HTML: &str = "
<!DOCTYPE html>
<html>
    <head>
        <title>Discord Authorization Successful</title>
    </head>
    <body>
        <p>Awesome! You have successfully connected your Discord account with SimplyPlural2Any.</p>
        <p>You can now close this window now.</p>
    </body>
</html>
";

#[post("/api/user/platform/discord/pairing-code")]
pub async fn post_api_user_discord_pairing_code(
    db_pool: &State<PgPool>,
    jwt: users::Jwt,
    client: &State<reqwest::Client>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
) -> HttpResult<Json<String>> {
    let user_id = jwt.user_id()?;

    let code = generate_9_digit_apiring_code();

    database::modify_user_secrets(
        db_pool,
        &user_id,
        client,
        application_user_secrets,
        |user_with_secrets| {
            user_with_secrets.discord_pairing_code = Some(code.clone().into());
            user_with_secrets.discord_pairing_code_expires_at =
                Some(chrono::Utc::now() + chrono::Duration::minutes(5));
        },
    )
    .await?;

    Ok(Json(code))
}

#[post("/api/user/platform/discord/pair?<pairing_code>")]
pub async fn post_api_user_discord_pair(
    db_pool: &State<PgPool>,
    jwt: users::Jwt,
    pairing_code: String,
    application_user_secret: &State<database::ApplicationUserSecrets>,
) -> HttpResult<Json<String>> {
    let user_id = jwt.user_id()?;

    let _user_secrets =
        database::get_user_secrets(db_pool, &user_id, application_user_secret).await?;

    // todo. check: user_secrets.discord_pairing_code == request.pairing_code and that code in DB isn't expired in DB.

    let bridge_secret = generate_random_secret();

    eprintln!("{pairing_code}");

    // todo. unset discord_pairing code and it's expitation date
    // todo. set bridge_secret. and save all changes.

    Ok(Json(bridge_secret))
}

pub fn generate_random_secret() -> String {
    StdRng::from_os_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub fn generate_9_digit_apiring_code() -> String {
    let mut rng = rand::rngs::StdRng::from_os_rng();
    let code: u32 = rng.random_range(100_000_000..1_000_000_000);
    format!("{code:09}")
}
