use anyhow::Result;
use sp2any::{
    for_discord_bridge::UserLoginCredentials,
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
    let type_defs = vec![
        export::<Email>(conf)?,
        export::<UserProvidedPassword>(conf)?,
        export::<UserLoginCredentials>(conf)?,
        export::<JwtString>(conf)?,
        "export type Platform = \"VRChat\" | \"Discord\" | \"DiscordStatusMessage\"".to_owned(),
        "export type UpdaterStatus = \"Inactive\" | \"Running\" | { \"Error\": string }".to_owned(),
        "export type UserUpdatersStatuses = { [p in Platform]?: UpdaterStatus }".to_owned(),
    ];
    let types = type_defs.join("\n");
    fs::write(DESTINATION, types)?;
    println!("Done.");
    Ok(())
}
