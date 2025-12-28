# TODO

Always read `AGENTS.md` for general info on the project.

I want to add another feature to PluralSync so that a new feature is added: Users
should be able to configure to have the name/display-name of a PluralKit member used instead of the SimplyPlural name.
For members without PluralKit id, the fallback is simply the existing SimplyPlural name.

# STEPS

1.  **Database Migration:** Create a new SQL migration to add a `use_pluralkit_name` column to the `users` table. This will store an enum-like value (e.g., a string: 'NoOverride', 'UsePluralKitName', 'UsePluralKitDisplayName') and should default to 'NoOverride'.
2.  **User Configuration:**
    a. Define a `UsePluralKitName` enum in Rust (e.g., in `src/users/config.rs`). It should have `NoOverride`, `UsePluralKitName`, and `UsePluralKitDisplayName` variants.
    b. Add a `use_pluralkit_name` field of this enum type to the `UserConfigDbEntries` and `UserConfigForUpdater` structs in `src/users/config.rs`.
3.  **Data Model Expansion:** In `src/plurality/simply_plural_model.rs`:
    a. Add `pluralkit_name: Option<String>` and `pluralkit_display_name: Option<String>` to the `MemberContent` struct.
    b. Add the same two fields to the `Fronter` struct.
    c. Update the `From<Member> for Fronter` implementation to copy these new fields from the `Member`'s content to the `Fronter`.
4.  **PluralKit API Client:** Create a new file `src/plurality/pluralkit.rs`.
    a. Define a `PluralKitMember` struct to deserialize the API response, including `id: String`, `name: String`, and `display_name: Option<String>`.
    b. Implement a `pub async fn get_pluralkit_members(config: &UserConfigForUpdater) -> Result<HashMap<String, PluralKitMember>>`.
    c. This function will make an authenticated GET request to `https://api.pluralkit.me/v2/systems/@me/members`, following the pattern in `src/platforms/to_pluralkit.rs` (setting `Authorization` and `User-Agent` headers).
    d. It will deserialize the JSON response into a `Vec<PluralKitMember>` and convert it to a `HashMap` keyed by the PluralKit member `id` before returning. Empty-strings or white-space only strings will be not used.
5.  **Core Logic (Member Enrichment):** In `src/plurality/simply_plural.rs`, update `get_members_and_custom_fronters_by_privacy_rules`:
    a. If the user's `use_pluralkit_name` config is not `NoOverride`, call `pluralkit::get_pluralkit_members`.
    b. Iterate through the list of `Member` structs mutably. For each member that has a `pluralkit_id`, look it up in the `HashMap` of PluralKit members and populate the `member.content.pluralkit_name` and `member.content.pluralkit_display_name` fields.
6.  **Core Logic (Fronter Name Selection):**
    a. In `src/plurality/simply_plural_model.rs`, implement a new method for the `Fronter` struct: `pub fn name<'a>(&'a self, name_config: &UsePluralKitName) -> &'a str`. This method will select the name to use based on the config, with appropriate fallbacks: `UsePluralKitDisplayName` -> `pluralkit_display_name` (if not empty), else `pluralkit_name`, else SimplyPlural `name`. `UsePluralKitName` -> `pluralkit_name`, else SimplyPlural `name`. `NoOverride` -> SimplyPlural `name`.
    b. In `src/plurality/fronting_status.rs`, pass the `UserConfigForUpdater` down to `collect_clean_fronter_names`.
    c. In `collect_clean_fronter_names`, replace the call to `f.preferred_vrchat_status_name()` with a call to the new `f.name(&config.use_pluralkit_name)`. The existing VRChat name cleaning logic will be applied to the result of this new method.
7.  **Frontend UI (Config):** In `frontend/src/components/Config.vue`, replace the previous UI with a dropdown or radio button group to manage the `use_pluralkit_name` setting.
8.  **Testing:** Write unit and integration tests for the new PluralKit name override feature.


# Information


## Pluralkit API

### Get System Members

GET /systems/{systemRef}/members

Returns a list of member objects.

### Member model
key 	type 	notes
id 	string 	
uuid 	string 	
?system 	string 	id of system this member is registered in (only returned in /members/:id endpoint)
name 	string 	100-character limit
display_name 	?string 	100-character limit
color 	?string 	6-character hex code, no # at the beginning
birthday 	?string 	YYYY-MM-DD format, 0004 hides the year
pronouns 	?string 	100-character-limit
avatar_url 	?string 	256-character limit, must be a publicly-accessible URL
webhook_avatar_url 	?string 	256-character limit, must be a publicly-accessible URL
banner 	?string 	256-character limit, must be a publicly-accessible URL
description 	?string 	1000-character limit
created 	?datetime 	
proxy_tags 	array of ProxyTag objects 	
keep_proxy 	boolean 	
tts 	boolean 	
autoproxy_enabled 	?boolean 	
message_count 	?int 	
last_message_timestamp 	?datetime 	
privacy 	?member privacy object 	

Member privacy keys: visibility, name_privacy, description_privacy, birthday_privacy, pronoun_privacy, avatar_privacy, banner_privacy, metadata_privacy, proxy_privacy