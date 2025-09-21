use anyhow::Result;
use sp2any::{
    database::Decrypted, for_discord_bridge::UserLoginCredentials, license, users::{Email, JwtString, UserProvidedPassword}
};
use specta::{
    self,
    ts::{ExportConfiguration, export},
};
use std::fs;

const DESTINATION: &str = "./frontend/src/sp2any.bindings.ts";

fn main() -> Result<()> {
    println!("Exporting to {DESTINATION}...");
    let conf = &ExportConfiguration::default();
    let defs = [
        export::<Email>(conf)?,
        export::<UserProvidedPassword>(conf)?,
        export::<UserLoginCredentials>(conf)?,
        export::<Decrypted>(conf)?,
"export type UserConfigDbEntries = {
    wait_seconds?: number;
    system_name?: string;
    status_prefix?: string;
    status_no_fronts?: string;
    status_truncate_names_to?: number;
    enable_discord?: boolean;
    enable_discord_status_message?: boolean;
    enable_vrchat?: boolean;
    simply_plural_token?: Decrypted;
    discord_status_message_token?: Decrypted;
    vrchat_username?: Decrypted;
    vrchat_password?: Decrypted;
    vrchat_cookie?: Decrypted;
}".to_owned(),
        export::<JwtString>(conf)?,
        "export type Platform = \"VRChat\" | \"Discord\" | \"DiscordStatusMessage\"".to_owned(),
        "export type UpdaterStatus = \"Disabled\" | \"Running\" | { \"Error\": string }".to_owned(),
        "export type UserUpdatersStatuses = { [p in Platform]?: UpdaterStatus }".to_owned(),
        format!(
            "export const LICENSE_INFO_SHORT_HTML: string = \"{}\"",
            license::info_short_html().replace('"', "\\\"")
        ),
    ];
    fs::write(DESTINATION, defs.map(|s| s + ";").join("\n"))?;
    println!("Done.");
    Ok(())
}
