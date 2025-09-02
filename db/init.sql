CREATE EXTENSION IF NOT EXISTS pgcrypto;
/*
secrets fields are stored as encrypted bytea fields.
we use the users hash(uuid + application_user_secret) as the secret.
(specifically sha256(user_uuid, application_user_secret) has hex_string)
This secret is same for an individual users' fields, but different for each user.

WHEN THE SALT IS CHANGED, THEN WE NEED TO RE-ENCRYPT ALL ENCRYPTED VALUES! TAKE CARE!

*/

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    email VARCHAR(127) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    wait_seconds INTEGER CHECK (wait_seconds > 0),
    request_timeout INTEGER CHECK (request_timeout > 0),
    
    system_name TEXT,
    status_prefix TEXT,
    status_no_fronts TEXT,
    status_truncate_names_to INTEGER CHECK (status_truncate_names_to > 0),
    
    enable_discord BOOLEAN NOT NULL DEFAULT false,
    enable_discord_status_message BOOLEAN NOT NULL DEFAULT false,
    enable_vrchat BOOLEAN NOT NULL DEFAULT false,
    
    bridge_pairing_code_expires_at TIMESTAMPTZ,
    
    /* encrypted secrets. need to be re-encrypted, when password changes. */
    enc__simply_plural_token bytea,
    enc__discord_status_message_token bytea,
    enc__discord_user_id bytea,
    enc__discord_oauth_access_token bytea,
    enc__discord_oauth_refresh_token bytea,
    enc__bridge_pairing_code bytea,
    enc__bridge_secret bytea,
    enc__vrchat_username bytea,
    enc__vrchat_password bytea,
    enc__vrchat_cookie bytea

    /* constraints to check manually before inserting into db:
    whenever a platform is enabled, the corresponding fields must be not null.*/
);
