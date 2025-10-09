use anyhow::Result;
use sp2any::{
    database::Decrypted,
    platforms::{
        TwoFactorAuthCode, TwoFactorAuthMethod, TwoFactorCodeRequiredResponse, VRChatCredentials,
        VRChatCredentialsWithCookie, VRChatCredentialsWithTwoFactorAuth,
    },
    updater::Platform,
    users::PrivacyFineGrained,
};
use sp2any_base::{
    license,
    meta::{
        CANONICAL_SP2ANY_BASE_URL, SP2ANY_GITHUB_REPOSITORY_RELEASE_ASSETS_URL, SP2ANY_VERSION,
        SP2AnyVariantInfo,
    },
    users::{Email, JwtString, UserLoginCredentials, UserProvidedPassword},
};
use specta::ts::{ExportConfiguration, export};
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
        export::<SP2AnyVariantInfo>(conf)?,
        format!("export const CANONICAL_SP2ANY_BASE_URL: string = \"{CANONICAL_SP2ANY_BASE_URL}\""),
        format!("export const SP2ANY_GITHUB_REPOSITORY_RELEASE_ASSETS_URL: string = \"{SP2ANY_GITHUB_REPOSITORY_RELEASE_ASSETS_URL}\""),
"export type UserConfigDbEntries = {
    wait_seconds?: number;
    website_system_name?: string;
    website_url_name?: string;
    status_prefix?: string;
    status_no_fronts?: string;
    status_truncate_names_to?: number;
    privacy_fine_grained?: PrivacyFineGrained;
    privacy_fine_grained_buckets?: string[];
    show_members_non_archived?: boolean;
    show_members_archived?: boolean;
    show_custom_fronts?: boolean;
    respect_front_notifications_disabled?: boolean;
    enable_website?: boolean;
    enable_discord?: boolean;
    enable_discord_status_message?: boolean;
    enable_vrchat?: boolean;
    simply_plural_token?: Decrypted;
    discord_status_message_token?: Decrypted;
    vrchat_username?: Decrypted;
    vrchat_password?: Decrypted;
    vrchat_cookie?: Decrypted;
}".to_owned(),
        export::<PrivacyFineGrained>(conf)?,
        export::<JwtString>(conf)?,
        export::<Platform>(conf)?,
        "export type UpdaterStatus = \"Disabled\" | \"Running\" | { \"Error\": string } | \"Starting\"".to_owned(),
        "export type UserUpdatersStatuses = { [p in Platform]?: UpdaterStatus }".to_owned(),
        format!(
            "export const LICENSE_INFO_SHORT_HTML: string = \"{}\"",
            license::info_short_html().replace('"', "\\\"")
        ),
        export::<VRChatCredentials>(conf)?,
        export::<VRChatCredentialsWithCookie>(conf)?,
        export::<TwoFactorAuthMethod>(conf)?,
        export::<TwoFactorCodeRequiredResponse>(conf)?,
        export::<TwoFactorAuthCode>(conf)?,
        export::<VRChatCredentialsWithTwoFactorAuth>(conf)?,
        format!("export const SP2ANY_VERSION = \"{SP2ANY_VERSION}\""),
        "export type VRChatAuthResponse = { Left: VRChatCredentialsWithCookie } | { Right: TwoFactorCodeRequiredResponse }".to_owned(),
    ];
    fs::write(DESTINATION, defs.map(|s| s + ";").join("\n"))?;
    println!("Done.");
    Ok(())
}
