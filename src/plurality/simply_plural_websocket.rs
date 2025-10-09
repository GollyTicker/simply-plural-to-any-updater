use anyhow::Result;
use futures::{self, SinkExt, StreamExt};
use serde_json::{self};
use std::time::Duration;
use tokio::{net::TcpStream, select};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Utf8Bytes, protocol::Message},
};

const WEBSOCKET_URL: &str = "wss://api.apparyllis.com/v1/socket";
const RETRY_WAIT_SECONDS: u64 = 10;
const KEEP_ALIVE_INTERVAL: u64 = 30;

type WriteStream = futures::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type ReadStream = futures::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/**
 * Simplifies the handling of messages from Simpl Plural.
 *
 * Takes an message handler, and applies that handler to each event message from simply plural.
 *
 * This function takes care of the responsiblities of connection, re-connection, authentication, aborting on failures, etc.
 */
#[allow(clippy::future_not_send)]
pub async fn auto_reconnecting_websocket_client_to_simply_plural<F>(
    log_prefix: &str,
    token: &str,
    process_event: impl Fn(Utf8Bytes) -> F,
) -> !
where
    F: Future<Output = Result<()>>,
{
    loop {
        log::info!("WS {log_prefix} client starting ...");
        if let Err(e) = run_single_websocket_connection(log_prefix, token, &process_event).await {
            log::error!("WS {log_prefix} client error: {e}.",);
        } else {
            log::warn!("WS {log_prefix} client exited cleanly, which should not happen.",);
        }

        log::info!("WS {log_prefix} Retrying in {RETRY_WAIT_SECONDS} seconds...");
        tokio::time::sleep(Duration::from_secs(RETRY_WAIT_SECONDS)).await;
    }
}

async fn run_single_websocket_connection<F>(
    log_prefix: &str,
    token: &str,
    process_event: impl Fn(Utf8Bytes) -> F,
) -> Result<()>
where
    F: Future<Output = Result<()>>,
{
    let (mut write, mut read) = create_connection(log_prefix, WEBSOCKET_URL).await?;

    authenticate(log_prefix, token, &mut write).await?;

    let mut keep_alive_interval = tokio::time::interval(Duration::from_secs(KEEP_ALIVE_INTERVAL));

    loop {
        select! {
            Some(msg) = read.next() => {
                match msg? {
                    Message::Text(pong) if pong == "pong" => log::info!("SP WS Ok Pong."),
                    Message::Text(json_string) => {
                        log::info!("WS {log_prefix} Received payload: '{json_string}'");
                        process_event(json_string).await?;
                    },
                    Message::Close(_) => return Ok(()),
                    unknown => log::warn!("WS {log_prefix} Unknown message '{unknown}'. Ignoring and continueing."),
                }
            }
            _ = keep_alive_interval.tick() => {
                write.send(Message::Text("ping".into())).await?;
                log::info!("WS {log_prefix} Ping sent.");
            }
        }
    }
}

async fn authenticate(log_prefix: &str, token: &str, write: &mut WriteStream) -> Result<()> {
    log::info!("WS {log_prefix} client authenticating...");
    let auth_payload = serde_json::json!({
        "op": "authenticate",
        "token": token,
    });
    write
        .send(Message::Text(auth_payload.to_string().into()))
        .await?;
    log::info!("WS {log_prefix} client authentication sent.");
    Ok(())
}

async fn create_connection(log_prefix: &str, url: &str) -> Result<(WriteStream, ReadStream)> {
    log::info!("WS {log_prefix} client connecting to WebSocket: {url}");
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
    let (write, read) = ws_stream.split();
    Ok((write, read))
}
