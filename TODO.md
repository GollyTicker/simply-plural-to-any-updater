# TODO

We want to add support for privacy bucket based fine-grained control adn visibility of fronts.

For that, I've already added the `privacy_buckets` fields in `simply_plural_model.rs`. I've also already
added the new configuration values for the UI in `Config.vue`.

You can fetch a friend's buckets via GET https://api.apparyllis.com/v1/friend/<system-id>/<friend-id>. The simply_plural_model.rs
already contains the response structs and the GLOBAL_SP2ANY_ON_SIMPLY_PLURAL_USER_ID.

Now it's time to implement this all. Outline all the steps you need to do.

# STEPS

1.  DONE - **Backend - `src/users/config.rs`:**
    *   Define a `PrivacyFineGrained` enum with variants `NoFineGrained`, `ViaFriend`, and `ViaPrivacyBuckets`.
    *   Derive `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`, `Eq`, `Default`, `sqlx::Type`, and `specta::Type` for the enum.
    *   Annotate the enum with `#[specta(export)]` and `#[sqlx(type_name = "privacy_fine_grained_enum")]`.
    *   Update the `UserConfigDbEntries` struct to use `privacy_fine_grained: Option<PrivacyFineGrained>`.
    *   Add `privacy_fine_grained_buckets: Option<Vec<String>>` to the `UserConfigDbEntries` struct.
    *   Update the `default()` and `with_defaults()` methods to handle the new fields.

2.  **Database Migration:**
    *   DONE - Create a new migration file `docker/migrations/005_member_privacy_buckets.sql`.
    *   DONE - In this file, first create a new ENUM type: `CREATE TYPE privacy_fine_grained_enum AS ENUM ('NoFineGrained', 'ViaFriend', 'ViaPrivacyBuckets');`
    *   DONE - Then, add the following columns to the `users` table. We'll use a native PostgreSQL array type for the bucket IDs.
        *   DONE - `privacy_fine_grained` (`privacy_fine_grained_enum`) - This will store the enum value.
        *   DONE - `privacy_fine_grained_buckets` (TEXT[])

3.  DONE - **Backend - `src/database/queries.rs`:**
    *   Update the `update_user_config` query to include the new `privacy_fine_grained` and `privacy_fine_grained_buckets` columns. `sqlx` will automatically handle mapping `Vec<String>` to the `TEXT[]` column type.
    *   Update the `get_user_config` query to retrieve the new columns.

4.  DONE **Backend - `src/plurality/simply_plural.rs`:**
    *   Locate the fronter filtering logic (where `config.show_members_non_archived` is used).
    *   Implement the new filtering logic based on `config.privacy_fine_grained`:
        *   **`PrivacyFineGrained::ViaFriend`:**
            *   Fetch the user's own Simply Plural user ID.
            *   Fetch the friend details for the SP2Any user using `GET /v1/friend/<system-id>/<sp2any-friend-id>`, where `<sp2any-friend-id>` is `GLOBAL_SP2ANY_ON_SIMPLY_PLURAL_USER_ID`.
            *   The response will contain the `assigned_privacy_buckets`.
            *   Filter members and custom fronts, showing only those whose `privacy_buckets` list contains at least one of the `assigned_privacy_buckets`.
        *   **`PrivacyFineGrained::ViaPrivacyBuckets`:**
            *   Parse the comma-separated list of bucket IDs from `config.privacy_fine_grained_buckets`.
            *   Filter members/custom fronts, showing only those that are in one of the selected buckets.
        *   **`PrivacyFineGrained::NoFineGrained` or `None`:**
            *   The existing logic should apply.

5.  DONE **Backend - `src/bin/ts-bindings.rs`:**
    *   Ensure that the new enum and fields in `UserConfig` are correctly exported to TypeScript. This should be automatic if `specta` is used correctly.

6.  DONE **Frontend - `frontend/src/components/Config.vue`:**
    *   The component seems mostly ready.

# DOCUMENTATION
