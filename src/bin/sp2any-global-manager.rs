use anyhow::{Result, anyhow};
use futures::{self, SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{self};
use sp2any::setup;
use std::env;
use std::time::Duration;
use tokio::{net::TcpStream, select};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::protocol::Message};

const WEBSOCKET_URL: &str = "wss://api.apparyllis.com/v1/socket";
const RETRY_WAIT_SECONDS: u64 = 10;
const KEEP_ALIVE_INTERVAL: u64 = 30;

type WriteStream = futures::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type ReadStream = futures::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

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

    loop {
        // DO NOT USE the ? operator in this loop, as otherwise the loop will break!
        log::info!("WS client starting ...");
        if let Err(e) = run_websocket_client(&token).await {
            log::error!("WS client error: {e}.",);
        } else {
            log::warn!("WS client exited cleanly, which should not happen.",);
        }

        log::info!("Retrying in {RETRY_WAIT_SECONDS} seconds...");
        tokio::time::sleep(Duration::from_secs(RETRY_WAIT_SECONDS)).await;
    }
}

async fn handle_websocket_event(token: &str, event: Event<'_>) -> Result<()> {
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
    let client = reqwest::Client::new();

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

async fn run_websocket_client(token: &str) -> Result<()> {
    let (mut write, mut read) = create_connection(WEBSOCKET_URL).await?;

    authenticate(&mut write, token).await?;

    let mut keep_alive_interval = tokio::time::interval(Duration::from_secs(KEEP_ALIVE_INTERVAL));

    loop {
        select! {
            Some(msg) = read.next() => {
                match msg? {
                    Message::Text(pong) if pong == "pong" => log::info!("Ok Pong."),
                    Message::Text(json_string) => {
                        log::info!("Received payload: '{json_string}'");
                        let event = serde_json::from_str::<Event>(&json_string)?;
                        handle_websocket_event(token, event).await?;
                    },
                    Message::Close(_) => return Ok(()),
                    unknown => log::warn!("Unknown message '{unknown}'. Ignoring and continueing."),
                }
            }
            _ = keep_alive_interval.tick() => {
                write.send(Message::Text("ping".into())).await?;
                log::info!("Ping sent.");
            }
        }
    }
}

async fn authenticate(write: &mut WriteStream, token: &str) -> Result<()> {
    log::info!("WS client authenticating...");
    let auth_payload = serde_json::json!({
        "op": "authenticate",
        "token": token,
    });
    write.send(Message::Text(auth_payload.to_string().into())).await?;
    log::info!("WS client authentication sent.");
    Ok(())
}

async fn create_connection(url: &str) -> Result<(WriteStream, ReadStream)> {
    log::info!("WS client connecting to WebSocket: {url}");
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
    let (write, read) = ws_stream.split();
    Ok((write, read))
}
