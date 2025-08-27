use std::{error::Error, time::Duration};

use chrono::Utc;
use discord_rich_presence::{
    activity::{self, ActivityType, Assets, Button, Party, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use serde::Deserialize;
use tokio::time::sleep;

const UPDATE_REPO_OWNER: &str = "GollyTicker";
const UPDATE_REPO_NAME: &str = "simply-plural-to-any-updater";
const UPDATE_BIN_NAME: &str = "sp2any-discord-bridge";

#[allow(clippy::unreadable_literal)]
const DISCORD_SP2ANY_BOT_APPLICATION_ID: u64 = 1408232222682517575;

const FRONTING_TEST_IMAGE: &str = "https://ayake.net/cloud/apps/files_sharing/publicpreview/wewER2MaZ4JbXEg?file=/&fileId=28035&x=3424&y=1926&a=true&etag=d150d19707ca3b6ef1470e0853bb7da7";

// todo. add auto-start capabilities: https://crates.io/crates/auto-launch
// todo. note, that only a single user account is supported for now.

#[tokio::main]
async fn main() {
    if needs_restart_after_automatic_update() {
        return;
    }

    loop {
        match connect_to_discord_ipc().await {
            Ok(mut client) => {
                let e = activity_loop(&mut client).await;
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

fn run_update() -> Result<self_update::Status, Box<dyn ::std::error::Error>> {
    let version = env!("DYNAMIC_VERSION");
    let mut binding = self_update::backends::github::Update::configure();
    let builder = binding
        .repo_owner(UPDATE_REPO_OWNER)
        .repo_name(UPDATE_REPO_NAME)
        .bin_name(UPDATE_BIN_NAME)
        .show_download_progress(true)
        .current_version(version);

    if version.contains('-') {
        // todo!()
        // builder = builder.prerelease(true);
        ();
    }

    let status = builder.build()?.update()?;
    Ok(status)
}

fn needs_restart_after_automatic_update() -> bool {
    eprintln!("Checking for updates...");
    match run_update() {
        Ok(status) => {
            if status.updated() {
                eprintln!("Update successful! New version: {}", status.version());
                eprintln!("Restarting after automatic update.");
                true
            } else {
                eprintln!("Already up to date.");
                false
            }
        }
        Err(e) => {
            eprintln!("Update failed: {e}");
            false
        }
    }
}

async fn activity_loop(client: &mut DiscordIpcClient) -> Box<dyn Error> {
    // todo. do startup and pairing protocol here
    loop {
        match set_activity_and_wait(client).await {
            Ok(()) => (),
            Err(e) => {
                return e;
            }
        }
    }
}

async fn connect_to_discord_ipc() -> Result<DiscordIpcClient, Box<dyn Error>> {
    eprintln!("creating client...");
    let mut client = DiscordIpcClient::new(&DISCORD_SP2ANY_BOT_APPLICATION_ID.to_string())?;
    eprintln!("created. connecting...");
    let ready: ReadyResponse = serde_json::from_value(client.connect()?)?;
    let user = ready.data.user;
    eprintln!("connected to user: {user:?}");
    Ok(client)
}

async fn set_activity_and_wait(client: &mut DiscordIpcClient) -> Result<(), Box<dyn Error>> {
    eprintln!("sending payload...");
    // note. tell users they may need to activate rich presence sharing in their activity privacy settings. they can also customize it per server.

    let activity_type = ActivityType::Playing; // display as rich presence!
                                               // visible on yourself as well as on others. but the button isn't available for everyone to see
                                               // OR
                                               // let activity_type = ActivityType::Custom; // display as custom status message!
                                               // only visible to yourself when you haven't set a custom status message manually AND when you are not hovering
                                               // over your status on the botom left. You can also not see it on your full bio lol.
                                               // however, it seems to be overshadowed by the normal custom status, if it's manually set by the user! to be noted!
                                               //what about hungstatus? and is the RPC method limited or does it work scalably??? Do I need to have it verified?
                                               // https://discord.com/developers/docs/topics/rpc
                                               // or is this already done by this create?
                                               // NOTE. THIS DOESN'T WORK WITH THE OFFICIAL DISCORD CLIENT! I can offer it, but let users know, that it only works with
                                               // certain modded clients and that there is no guarantee.

    // Formatting based on activity type: https://discord.com/developers/docs/events/gateway-events#activity-object-activity-types

    let payload = activity::Activity::new()
        .activity_type(activity_type)
        .timestamps(
            Timestamps::new()
                .start(Utc::now().timestamp() - 1000)
                .end(Utc::now().timestamp() + 1000),
        )
        .details("details: test F: Ayake, Felina, Hole")
        .state("state: test F: Ayake, Felina, Hole")
        .party(Party::new().id("party-id").size([3, 9]))
        .buttons(vec![Button::new(
            "View Online",
            "https://ayake.net/fronting",
        )]) // todo. maybe add a buttom to the fronting website?
        // .secrets(Secrets::new().spectate("some-sepctate-secret"))
        .assets(
            Assets::new()
                .small_image(FRONTING_TEST_IMAGE)
                .small_text("small Ayake Sparkle 💖")
                .large_image(FRONTING_TEST_IMAGE)
                .large_text("large Ayake Sparkle 💖"),
        );
    client.set_activity(payload)?;
    eprintln!("sent! waiting now.");

    sleep(Duration::from_secs(5)).await;
    // Status is shown as long as this RPC is running and is connected.

    Ok(())
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
