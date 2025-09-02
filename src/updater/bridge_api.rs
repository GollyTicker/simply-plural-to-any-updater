use crate::database;
use crate::http::HttpResult;
use crate::users;
use anyhow::anyhow;
use rand::Rng;
use rand::{distr::Alphanumeric, rngs::StdRng, SeedableRng};
use rocket::serde::json::Json;
use rocket::{response, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[post("/api/user/bridge/new-pairing-code")]
pub async fn post_api_user_bridge_new_pairing_code(
    db_pool: &State<PgPool>,
    jwt: users::Jwt,
    client: &State<reqwest::Client>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
) -> HttpResult<Json<String>> {
    let user_id = jwt.user_id()?;

    let code = generate_9_digit_pairing_code();

    database::modify_user_secrets(
        db_pool,
        &user_id,
        client,
        application_user_secrets,
        |user_with_secrets| {
            user_with_secrets.bridge_pairing_code = Some(code.clone().into());
            user_with_secrets.bridge_pairing_code_expires_at =
                Some(chrono::Utc::now() + chrono::Duration::minutes(5));
        },
    )
    .await?;

    Ok(Json(code))
}

#[derive(Deserialize)]
pub struct BridgePairingRequest {
    pub discord_user_id: String,
    pub pairing_code: String,
}

#[derive(Serialize)]
pub struct BridgePairingResponse {
    pub bridge_secret: String,
}

// todo. we can make this bridge work later independent of discord. this is useful when setting the status for VRChat via OSC etc.
#[post("/api/user/bridge/pair", data = "<bridge_pairing_request>")]
pub async fn post_api_user_bridge_pair(
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
    bridge_pairing_request: Json<BridgePairingRequest>,
) -> HttpResult<Json<BridgePairingResponse>> {
    let user_id = find_user_by_discord_id_and_pairing_code(
        db_pool,
        application_user_secrets,
        &bridge_pairing_request.discord_user_id,
        &bridge_pairing_request.pairing_code,
    )
    .await?;
    let bridge_secret = generate_random_secret();

    database::modify_user_secrets(
        db_pool,
        &user_id,
        client,
        application_user_secrets,
        |user_with_secrets| {
            user_with_secrets.bridge_secret = Some(bridge_secret.clone().into());
            user_with_secrets.bridge_pairing_code = None;
            user_with_secrets.bridge_pairing_code_expires_at = None;
        },
    )
    .await?;

    Ok(Json(BridgePairingResponse { bridge_secret }))
}

// refactor the choice of picking a user (until THIS-POINT) into a separate function:
async fn find_user_by_discord_id_and_pairing_code(
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    discord_user_id: &str,
    pairing_code: &str,
) -> HttpResult<users::UserId> {
    let all_users = database::get_all_users(db_pool).await?;

    for user_id in all_users {
        let user_secrets =
            database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;

        let pairing_code_matches = user_secrets
            .bridge_pairing_code
            .as_ref()
            .is_some_and(|c| c.secret == pairing_code);

        let discord_id_matches = user_secrets
            .discord_user_id
            .as_ref()
            .is_some_and(|id| id.secret == discord_user_id);

        let code_not_expired = user_secrets
            .bridge_pairing_code_expires_at
            .is_some_and(|expires_at| expires_at > chrono::Utc::now());

        if pairing_code_matches && discord_id_matches && code_not_expired {
            return Ok(user_id);
        }
    }
    Err(response::Debug(anyhow!(
        "discord id doesn't match, or pairing code doesn't match or is expired."
    )))
}

fn generate_random_secret() -> String {
    StdRng::from_os_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn generate_9_digit_pairing_code() -> String {
    let mut rng = rand::rngs::StdRng::from_os_rng();
    let code: u32 = rng.random_range(100_000_000..1_000_000_000);
    format!("{code:09}")
}
