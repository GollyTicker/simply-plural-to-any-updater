
use crate::communication::HttpResult;
use crate::platforms::discord;
use crate::users::JwtString;
use crate::{database, updater, users};
use anyhow::{Result, anyhow};
use reqwest::Client;
use rocket::futures::{SinkExt, StreamExt};
use rocket::response::content::RawHtml;
use rocket::{State, response};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

use rocket_ws as ws;

const DEV_OAUTH_REDIRECT_URL: &str =
    "http://localhost:8080/api/user/platform/discord/oauth/callback";

// todo. this seems obsolete now?
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

/// This websocket stream sends text messages of the type DiscordRichPresence and
/// receives messages of the type UpdaterStatus.
#[allow(clippy::needless_pass_by_value)]
#[get("/api/user/platform/discord/bridge-events")]
pub async fn get_api_user_platform_discord_bridge_events(
    jwt: users::Jwt,
    ws: ws::WebSocket,
    shared_updaters: &State<updater::UpdaterManager>,
    db_pool: &State<PgPool>,
    client: &State<reqwest::Client>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
) -> Result<ws::Stream!['static], response::Debug<anyhow::Error>> {
    let user_id = jwt.user_id()?;
    let config = database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;
    let (config, _) = users::create_config_with_strong_constraints(&user_id, client, &config)?;

    // todo. if discord is not enabled, then send the appropriate flags.

    let mut fronting_channel = shared_updaters.subscribe_fronter_channel(&user_id)?;

    // todo. add channel here to finish propagating the statuses to the shared updater and then to the UI.
    // let notify_status = move |user_id: &UserId, s: updater::UpdaterStatus| -> Result<()> {
    //     let mut map = HashMap::new();
    //     map.insert(updater::Platform::Discord, s);
    //     shared_updaters.notify_updater_statuses(user_id, map)
    // };

    let stream = {
        ws::Stream! { ws =>
            let mut ws = ws.fuse();
            let user_id = user_id.clone();

            loop {
                tokio::select! {
                    fronters_msg = fronting_channel.recv() => {
                        match fronters_msg {
                            Some(fronters) => {
                                let rich_presence_result = discord::render_fronts_to_discord_rich_presence(fronters, &config);
                                match rich_presence_result {
                                    Ok(rich_presence) => {
                                        eprintln!("{user_id}: Sending rich presence to bridge via WebSocket...");
                                        let payload = match rocket::serde::json::to_string(&rich_presence) {
                                            Ok(p) => p,
                                            Err(e) => {
                                                eprintln!("{user_id}: Failed to serialize rich presence: {e}");
                                                continue;
                                            }
                                        };
                                        // notify_status(&user_id, updater::UpdaterStatus::Running);
                                        yield ws::Message::Text(payload);
                                    }
                                    Err(err) => {
                                        eprintln!(
                                            "{user_id}: Error while rendering fronts for discord rich presence. Continuing nonetheless. {err}"
                                        );
                                    }
                                }
                            },
                            None => break, // shared updater closed?
                        }
                    },
                    message = ws.next() => {
                        match message {
                            Some(Ok(ws::Message::Close(_))) => {
                                eprintln!("ended ws stream");
                                break;
                            },
                            Some(Ok(message)) => {
                                eprintln!("what is this? {message:?}");
                            },
                            Some(Err(s)) => {
                                eprintln!("some error: {s:?}");
                                break
                            }, // client disconnected
                            None => break, // client disconnected
                        }
                    }
                }
            }
            eprintln!("{user_id}: Ended WebSocket stream.");
        }
    };

    Ok(stream)
}
