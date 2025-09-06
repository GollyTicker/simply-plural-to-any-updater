use crate::{plurality, users};
use anyhow::Result;

pub struct DiscordUpdater {
    pub last_operation_error: Option<String>,
}
impl Default for DiscordUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscordUpdater {
    #[must_use]
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
        &self,
        _config: &users::UserConfigForUpdater,
        _fronts: &[plurality::Fronter],
    ) -> Result<()> {
        // todo. we want to control the http requests from the rich resence here and have that status here!
        /*
        TODO: The bridge (where the user is also logged in) connects to the local discord RPC and gets the local logged in user-id.
        DONE: The bridge regularly requests the newest rich presence data to be shown from our server.
            * the bridge always sends user auth for authentication
            * our server checks the auth and sends the new rich presence data
        */

        // it seems that we'll have to use a local RPC. I don't see any other option to do this via the remote bot API.
        // We'll have to make a small program locally which runs on the desktops with the users discord clients
        // AND which supports auto-start (on PC start) AND where users can connect to the SP2Any website.

        Ok(())
    }
}

/*
It's probably easier to simply request the user to directly login into the bridge
rather than doing some complicated pairing protocol which might also be harder to do correctly.

Once a bridge is authenticated, we can simply trust it to do stuff for us.

*/
