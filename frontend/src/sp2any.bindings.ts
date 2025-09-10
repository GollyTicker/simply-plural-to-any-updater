export type Email = { inner: string }
export type UserProvidedPassword = { inner: string }
export type UserLoginCredentials = { email: Email; password: UserProvidedPassword }
export type JwtString = { inner: string }
export type Platform = "VRChat" | "Discord" | "DiscordStatusMessage"
export type UpdaterStatus = "Inactive" | "Running" | { "Error": string }
export type UserUpdatersStatuses = { [p in Platform]?: UpdaterStatus }