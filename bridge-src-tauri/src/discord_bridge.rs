use std::time::Duration;

use anyhow::Result;
use discord_rich_presence::{
    activity::{Assets, Button, Party},
    DiscordIpc, DiscordIpcClient,
};
use serde::Deserialize;
use sp2any::platforms::{self, DiscordRichPresence};
use tokio::{
    sync::broadcast::{self},
    time::sleep,
};

// note. tell users they may need to activate rich presence sharing in their activity privacy settings. they can also customize it per server.

#[allow(clippy::unreadable_literal)]
const DISCORD_SP2ANY_BOT_APPLICATION_ID: u64 = 1408232222682517575;

pub async fn discord_ipc_loop(channel: broadcast::Sender<platforms::DiscordRichPresence>) {
    loop {
        match connect_to_discord_ipc() {
            Ok(mut client) => {
                let e = activity_loop(&mut client, channel.clone()).await;
                eprintln!("Activity loop ended with error: {e}");
                eprintln!("Reconnecting in 5s...");
                sleep(Duration::from_secs(5)).await;
            }
            Err(e) => {
                eprintln!("Error when connecting: {e}");
                eprintln!("Retrying in 5s...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn activity_loop(
    client: &mut DiscordIpcClient,
    channel: broadcast::Sender<platforms::DiscordRichPresence>,
) -> anyhow::Error {
    let mut receiver = channel.subscribe();
    loop {
        let update_result = match receiver.recv().await {
            Ok(discord_presence) => set_activity(client, discord_presence).await,
            Err(broadcast::error::RecvError::Closed) => clear_activity(client).await,
            Err(broadcast::error::RecvError::Lagged(_)) => Ok(()),
        };

        match update_result {
            Ok(()) => (),
            Err(e) => {
                return e;
            }
        }
    }
}

async fn set_activity(
    client: &mut DiscordIpcClient,
    discord_presence: platforms::DiscordRichPresence,
) -> Result<()> {
    let DiscordRichPresence {
        details,
        state,
        large_image_url,
        large_image_text,
        small_image_url,
        small_image_text,
        party_current,
        party_max,
        button_label,
        button_url,
    } = discord_presence.clone();

    let mut activity = discord_rich_presence::activity::Activity::new();
    activity = activity.details(details.as_str());
    activity = activity.state(state.as_str());

    let mut assets = Assets::new();
    if let Some(url) = large_image_url.as_ref() {
        assets = assets.large_image(url);
    }
    if let Some(text) = large_image_text.as_ref() {
        assets = assets.large_text(text);
    }
    if let Some(url) = small_image_url.as_ref() {
        assets = assets.small_image(url);
    }
    if let Some(text) = small_image_text.as_ref() {
        assets = assets.small_text(text);
    }
    activity = activity.assets(assets);

    if let (Some(party_current), Some(party_max)) = (party_current, party_max) {
        activity = activity.party(Party::new().size([party_current, party_max]));
    } else if let Some(party_current) = party_current {
        activity = activity.party(Party::new().size([party_current, 0]));
    }
    if let (Some(button_label), Some(button_url)) = (button_label.as_ref(), button_url.as_ref()) {
        activity = activity.buttons(vec![Button::new(button_label, button_url)]);
    }

    eprintln!("Setting activity: {discord_presence:?}");

    let () = client.set_activity(activity)?;

    Ok(())
}

async fn clear_activity(client: &mut DiscordIpcClient) -> Result<()> {
    eprintln!("Clearing activity ...");
    let () = client.clear_activity()?;
    Ok(())
}

fn connect_to_discord_ipc() -> Result<DiscordIpcClient> {
    eprintln!("creating client...");
    let mut client = DiscordIpcClient::new(&DISCORD_SP2ANY_BOT_APPLICATION_ID.to_string());
    eprintln!("created. connecting...");
    let ready: ReadyResponse = serde_json::from_value(client.connect()?)?;
    let user = ready.data.user;
    eprintln!("connected to user: {user:?}");
    Ok(client)
}

#[derive(Clone, Deserialize, Debug)]
struct ReadyResponse {
    pub data: ReadyResponseData,
}

#[derive(Clone, Deserialize, Debug)]
struct ReadyResponseData {
    pub user: DiscordUser,
}

#[derive(Clone, Deserialize, Debug)]
struct DiscordUser {
    pub id: String,
}
