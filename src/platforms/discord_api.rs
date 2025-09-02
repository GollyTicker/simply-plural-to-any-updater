use crate::http::HttpResult;
use crate::users::JwtString;
use crate::{database, plurality, users};
use anyhow::{anyhow, Result};
use reqwest::Client;
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::{response, State};
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

// --------------------------- TODO: CHECK CODE WHICH HAS BEEN AI-GENERATED -------------------------------

#[derive(Deserialize)]
pub struct BridgeActivityRequest {
    pub discord_user_id: String,
    pub bridge_secret: String,
}

#[derive(Serialize)]
pub struct BridgeActivityResponse {
    pub details: String,
    pub state: String,
    pub large_image_url: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_url: Option<String>,
    pub small_image_text: Option<String>,
    pub party_current: Option<i32>,
    pub party_max: Option<i32>,
    pub button_label: Option<String>,
    pub button_url: Option<String>,
}

#[get(
    "/api/user/platform/discord/fronting-for-rich-presence",
    data = "<request>"
)]
pub async fn get_api_platform_discord_fronting_for_rich_presence(
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
    request: Json<BridgeActivityRequest>,
) -> HttpResult<Json<BridgeActivityResponse>> {
    let all_users = database::get_all_users(db_pool).await?;

    for user_id in all_users {
        let user_secrets =
            database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;

        let discord_id_matches = user_secrets
            .discord_user_id
            .as_ref()
            .is_some_and(|id| id.secret == request.discord_user_id);

        let bridge_secret_matches = user_secrets
            .bridge_secret
            .as_ref()
            .is_some_and(|s| s.secret == request.bridge_secret);

        if discord_id_matches && bridge_secret_matches {
            let (config, _) =
                users::create_config_with_strong_constraints(&user_id, client, &user_secrets)?;

            let fronts = plurality::fetch_fronts(&config).await?;

            let fronting_format = plurality::FrontingFormat {
                max_length: None,
                cleaning: plurality::CleanForPlatform::NoClean,
                prefix: config.status_prefix.clone(),
                status_if_no_fronters: config.status_no_fronts.clone(),
                truncate_names_to_length_if_status_too_long: 1, // todo.
            };
            let status_string = plurality::format_fronting_status(&fronting_format, &fronts);

            let (large_image_url, large_image_text) = if fronts.len() == 1 {
                (
                    Some(fronts[0].avatar_url.clone()),
                    Some(fronts[0].name.clone()),
                )
            } else {
                (None, None)
            };

            let response = BridgeActivityResponse {
                details: status_string.clone(),
                state: status_string,
                large_image_url,
                large_image_text,
                small_image_url: None,
                small_image_text: None,
                party_current: Some(fronts.len() as i32),
                party_max: None,
                button_label: Some("View Fronters".to_string()),
                button_url: Some(format!("/api/fronting/{user_id}")),
            };

            return Ok(Json(response));
        }
    }

    Err(response::Debug(anyhow!(
        "Invalid bridge secret or Discord user ID."
    )))
}
