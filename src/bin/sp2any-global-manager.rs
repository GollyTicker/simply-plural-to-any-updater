use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde_json::{self};
use sp2any::{plurality, setup};
use std::env;
use tokio_tungstenite::tungstenite;

/** The Message as sent by Simply Plural on the Websocket.
 *
 * We use &str to make the code for parsing look better and simpler by being able to match against &str literals.
*/
#[derive(Debug, Clone, Deserialize)]
struct Event<'a> {
    msg: Option<&'a str>,
    title: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize)]
struct FriendRequest {
    #[serde(rename = "id")]
    from_user_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    setup::logging_init();

    #[allow(clippy::unwrap_used)]
    let token = env::var("GLOBAL_SP2ANY_SIMPLY_PLURAL_READ_WRITE_ADMIN_TOKEN")?;

    log::info!("Accept all friend requests before starting loop.");
    accept_all_friend_requests(&token).await?;

    plurality::auto_reconnecting_websocket_client_to_simply_plural("global-mgr", &token, |ev| {
        process_event(&token, ev)
    })
    .await
}

async fn process_event(token: &str, json_string: tungstenite::Utf8Bytes) -> Result<()> {
    let event = serde_json::from_str::<Event>(&json_string)?;
    match event.msg {
        None => log::info!("Ok empty event."),
        Some("Successfully authenticated") => log::info!("Ok authenticated."),
        Some("Authentication violation: Token is missing or invalid. Goodbye :)") => {
            Err(anyhow!("Auth failed."))?;
        }
        _ => match event {
            Event {
                msg: Some("notification"),
                title: Some("Friend request received"),
            } => {
                log::info!("Friend request received.");
                accept_all_friend_requests(token).await?;
            }
            _ => log::info!("Ignoring irrelevant event: {event:?}"),
        },
    }

    Ok(())
}

const ACCEPT_FRIEND_REQUEST_SETTINGS: &str =
    r#"{"settings":{"seeMembers": false, "seeFront": false, "getFrontNotif": false}}"#;

async fn accept_all_friend_requests(token: &str) -> Result<()> {
    log::info!("Fetching all friend requests and accepting them...");
    let client = setup::make_client()?;

    let incoming_requests_url = "https://api.apparyllis.com/v1/friends/requests/incoming";
    let friend_requests: Vec<FriendRequest> = client
        .get(incoming_requests_url)
        .header("Authorization", token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    for friend_request in &friend_requests {
        let url = format!(
            "https://api.apparyllis.com/v1/friends/request/respond/{}?accepted=true",
            friend_request.from_user_id
        );
        let _ = client
            .post(&url)
            .header("Authorization", token)
            .header("Content-Type", "application/json")
            .body(ACCEPT_FRIEND_REQUEST_SETTINGS)
            .send()
            .await?
            .error_for_status()?;
    }

    log::info!("All {} friend requests accepted.", friend_requests.len());

    Ok(())
}
