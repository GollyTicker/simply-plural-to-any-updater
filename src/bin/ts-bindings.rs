use anyhow::Result;
use sp2any::{
    for_discord_bridge::UserLoginCredentials,
    license,
    users::{Email, JwtString, UserProvidedPassword},
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
    let defs = vec![
        export::<Email>(conf)?,
        export::<UserProvidedPassword>(conf)?,
        export::<UserLoginCredentials>(conf)?,
        export::<JwtString>(conf)?,
        "export type Platform = \"VRChat\" | \"Discord\" | \"DiscordStatusMessage\"".to_owned(),
        "export type UpdaterStatus = \"Inactive\" | \"Running\" | { \"Error\": string }".to_owned(),
        "export type UserUpdatersStatuses = { [p in Platform]?: UpdaterStatus }".to_owned(),
        format!(
            "export const LICENSE_INFO_SHORT_HTML: string = \"{}\"",
            license::info_short_html().replace("\"", "\\\"")
        ),
    ];
    fs::write(DESTINATION, defs.join("\n"))?;
    println!("Done.");
    Ok(())
}
