use crate::{plurality, users};
use anyhow::Result;

#[allow(clippy::unreadable_literal)]
const DISCORD_SP2ANY_BOT_APPLICATION_ID: u64 = 1408232222682517575;

pub struct DiscordUpdater {
    pub last_operation_error: Option<String>,
}
impl DiscordUpdater {
    pub const fn new() -> Self {
        Self {
            last_operation_error: None,
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn setup(&self, _config: &users::UserConfigForUpdater) -> Result<()> {
        Ok(())
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &users::UserConfigForUpdater,
        fronts: &[plurality::Fronter],
    ) -> Result<()> {
/*
1. User goes to the SP2Any website and clicks on 'Authorize with Discord' to connect with discord.
2. Our server does OAuth flow with the user with `identify` scope and gets the discord-user-id.
    https://discord.com/developers/docs/topics/oauth2
    * our server is now trusted to decide what discord rich presence should show.
3. User downloads the SP2Any Discord Bridge and runs it.
4. The discord bridge
    4.1. connects to the local discord RPC and gets the local logged in user-id
    4.2. connects to our server and let's the server know, that it can set rich presence for this user.
5. Our server generates a 9 digit short-lived pairing code and shows it to the user.
6. User enters the same 9 digit code into the bridge and hence establishes trust between the bridge and our server.
7. Our webserver generates a secret which it stores with this user in the database and also sends to the bridge.
8. The bridge stores this secret locally.
9. The bridge regularly requests the newest rich presence data to be shown from our server.
    * the bridge always sends received secret for authentication
    * our server checks this secret for the claimed user-id by the bridge, and
        sends the new rich presence data, if the secret matches with the corresponding user's stored secret.
10. It is established, that only the sp2any discord bridge running on our users's computer is authorized to make such update requests.
        */

        // it seems that we'll have to use a local RPC. I don't see any other option to do this via the remote bot API.
        // We'll have to make a small program locally which runs on the desktops with the users discord clients
        // AND which supports auto-start (on PC start) AND where users can connect to the SP2Any website.

        Ok(())
    }
}
