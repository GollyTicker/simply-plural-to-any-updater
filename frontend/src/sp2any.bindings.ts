export type Email = { inner: string }
export type UserProvidedPassword = { inner: string }
export type UserLoginCredentials = { email: Email; password: UserProvidedPassword }
export type JwtString = { inner: string }
export type Platform = "VRChat" | "Discord" | "DiscordStatusMessage"
export type UpdaterStatus = "Disabled" | "Running" | { "Error": string }
export type UserUpdatersStatuses = { [p in Platform]?: UpdaterStatus }
export const LICENSE_INFO_SHORT_HTML: string = "<p class=\"license-short\"><a href=\"https://github.com/GollyTicker/simply-plural-to-any-updater\" target=\"_blank\">SP2Any</a> © 2025 by <a href=\"https://github.com/GollyTicker/\" target=\"_blank\">Ayake / GollyTicker</a> licensed Copyleft <a href=\"https://www.tldrlegal.com/license/gnu-affero-general-public-license-v3-agpl-3-0\" target=\"_blank\">AGPL</a></p>"