# TODO

Add functionality such that users can also synchronize their fronting status from SimplyPlural to PluralKit.

For that we need at least:
* adding the pk token as a new config field in the DB + a flag to enable/disable sync to pluralkit. See `docker/migrations` and `src/database/queries.rs`.
* extend simply plural JSON parsing by the new `pkId` field (only valid for members, not custom fronts). See `src/plurality/simply_plural_model.rs`.
* add the corresponding updater logic in the rust backend. See `src/updater` folder.
* extend the `Config.vue` in the `frontend`, so that people can see this new config and configure it
    * a note should be added, that syncing of the members doesn't happen automatically yet! We need to tackle that in a next step
* show the new updater in all the places where the existing ones are shown.

# STEPS

### Database Migration
1. DONE: Create a new SQL migration file: `docker/migrations/009_add_pluralkit_sync.sql`.
2. DONE: In this file, add a `pluralkit_token` secret encrypted field and a `enable_to_pluralkit` boolean flag to the `users` table.
3. DONE: Add the two new fields in the various config structs in `src/users` folder.

### Backend Implementation
1. DONE: In `src/database/queries.rs`, update the `User` struct and any relevant query functions to include the new `pluralkit_token` and `enable_to_pluralkit` fields. This will likely involve modifying `get_user` and `update_user` functions.
2. DONE: In `src/plurality/simply_plural_model.rs`, add an optional `pluralkit_id: Option<String>` field to the `Member` struct to hold the PluralKit member ID from Simply Plural. Rename it via serde, as the JSON field is called `pk_id` which is a bit less clear.
3. DONE: ImCreate a new module `src/platforms/pluralkit.rs`.
4. DONE: Inside `pluralkit.rs`, implement the core synchronization logic (analogous to how it's done for `vrchat.rs`):
    - Create a `PluralKitUpdater` struct. It should manage its own state, including `last_operation_error: Option<String>`.
    - Implement the `update_fronting_status` method. This method will receive the current fronters.
    - It will then find the fronter that has a `pluralkit_id`.
    - If a fronter with a `pluralkit_id` is found, it will make an API call to the PluralKit API to update the fronter:
        - **Endpoint:** `POST https://api.pluralkit.me/v2/systems/@me/switches`
        - **Headers:**
            - `Authorization`: The `pluralkit_token` from the user's config.
            - `Content-Type`: `application/json`.
            - `User-Agent`: A string identifying the application (e.g., "SimplyPlural-to-Any-Updater").
        - **Body:** A JSON object with a `members` key, containing a list of the `pluralkit_id`s of the current fronters. If no one is fronting, this should be an empty list.
    - The struct and its methods will be integrated into the `Updater` enum in `src/updater/platforms.rs`, similar to how `VRChatUpdater` is integrated.
5. DONE: In `src/updater/manager.rs` or `src/updater/mod.rs`, integrate the new `pluralkit` into the main updater loop or manager so it runs periodically.

#### Metrics
1.  **DONE: Define new metrics:** In `src/platforms/pluralkit.rs` (or a new `src/platforms/pluralkit_metrics.rs`), define the following metrics using the existing macros:
    *   `PLURAKIT_API_REQUESTS_TOTAL`: An `IntCounterVec` with labels for `user_id` and `status` (e.g., "success", "failure").
    *   `PLURAKIT_API_RATELIMIT_REMAINING`: An `IntGaugeVec` with labels for `user_id` and `scope`.

2.  **DONE: Implement metric recording:** In the `update_fronting_status` method in `pluralkit.rs`, after making the API call to PluralKit:
    *   Increment the `PLURAKIT_API_REQUESTS_TOTAL` counter with the appropriate status.
    *   Parse the `X-RateLimit-Remaining` and `X-RateLimit-Scope` headers from the HTTP response.
    *   Set the `PLURAKIT_API_RATELIMIT_REMAINING` gauge with the value of `X-RateLimit-Remaining` and the `scope` label.

3.  **DONE: Register new metrics:** In `src/metrics.rs`, register the new metrics in the `PROM_METRICS` static variable.

### Frontend Implementation
1. In `frontend/src/components/Config.vue`, add a new section for "PluralKit Synchronization".
2. Add a password input field for the "PluralKit Token" and bind it to the user's config object.
3. Add a checkbox to "Enable PluralKit Sync" and bind it to the user's config object.
4. Update the data-saving logic to include the new `pluralkit_token` and `enable_to_pluralkit` fields when sending updates to the backend.
5. The component `frontend/src/components/Status.vue` is responsible for displaying the status of all available updaters. It periodically calls the `sp2any_api.get_updater_status()` API endpoint, which returns a map of updater names to their statuses (`UserUpdatersStatuses`). The component then iterates over this map and displays each updater and its status. To have the "PluralKit Sync" updater appear, the backend needs to add it to the `UserUpdatersStatuses` map returned by the `get_updater_status` endpoint. No frontend changes are needed in `Status.vue` as it will dynamically render any updater provided by the backend.

# DOCUMENTATION

## Reading plural kit member id from SimplyPlural

The JSON we get from pluralkit for each member contains an optional string field `pkId`. That's the id of the member if they're synced with pluralkit.

e.g. `{ ..., "pkId": "pwhsdr", ... }`

## Creating a "switch" in PluralKit

Create Switch

POST /systems/{systemRef}/switches

JSON            Body                Parameters
key             type                description
?timestamp      datetime(1)         when the switch started
members         list of strings(2)  members present in the switch (or empty list for switch-out)

(1) Defaults to "now" when missing.

(2) Can be short IDs or UUIDs.

Returns a switch object containing a list of member objects.


## PluralKit Switch Object Model

Switch model
key         type                notes
id          uuid 	
timestamp   datetime 	
members     list of id/Member   Is sometimes in plain ID list form (eg. GET /systems/:id/switches), sometimes includes the full Member model (eg. GET /systems/:id/fronters).

## PluralKit API Reference

PluralKit has a basic HTTP REST API for querying and modifying your system. The root endpoint of the API is https://api.pluralkit.me/v2/.

### Authorization header token example

Authorization: z865MC7JNhLtZuSq1NXQYVe+FgZJHBfeBCXOPYYRwH4liDCDrsd7zdOuR45mX257

Endpoints will always return all fields, using null when a value is missing. On PATCH endpoints, missing fields from the JSON request will be ignored and preserved as is, but on POST endpoints will be set to null or cleared.

For models that have them, the keys id, uuid and created are not user-settable.

Endpoints taking JSON bodies (eg. most PATCH and PUT endpoints) require the Content-Type: application/json header set.

### User agent

The API requires the User-Agent header to be set to a non-empty string. Not doing so will return a 400 Bad Request with a JSON body.

If you are developing an application exposed to the public, we would appreciate if your User-Agent uniquely identifies your application, and (if possible) provides some contact information for the developers - so that we are able to contact you if we notice your application doing something it shouldn't.

### Authentication

Authentication is done with a simple "system token". You can get your system token by running pk;token using the Discord bot, either in a channel with the bot or in DMs. Then, pass this token in the Authorization HTTP header on requests that require it. Failure to do so on endpoints that require authentication will return a 401 Unauthorized.

Some endpoints show information that a given system may have set to private. If this is a specific field (eg. description), the field will simply contain null rather than the true value. If this applies to entire endpoint responses (eg. fronter, switches, member list), the entire request will return 403 Forbidden. Authenticating with the system's token (as described above) will override these privacy settings and show the full information.

### Rate Limiting

To protect against abuse and manage server resources, PluralKit's API limits the amount of queries available. Currently, the following limits are applied:

    10/second for any GET requests other than the messages endpoint (generic_get scope)
    10/second for requests to the Get Proxied Message Information endpoint (message scope)
    3/second for any POST, PATCH, or DELETE requests (generic_update scope)

We may raise the limits for individual users in a case-by-case basis; please ask in the support server

(opens new window) if you need a higher limit.

TIP

If you are looking to query a specific resource in your system repeatedly (polling), please consider using Dispatch Webhooks instead.

The following rate limit headers are present on HTTP responses:
name 	description
X-RateLimit-Limit 	The amount of total requests you have available per second.
X-RateLimit-Remaining 	The amount of requests you have remaining until the next reset time.
X-RateLimit-Reset 	The UNIX time (in milliseconds) when the ratelimit info will reset.
X-RateLimit-Scope 	The type of rate limit the current request falls under.

If you make more requests than you have available, the server will respond with a 429 status code and a JSON error body.

{
  "message": "429: too many requests",
  "retry_after": 19, // the amount of milliseconds remaining until you can make more requests
  "code": 0
}


