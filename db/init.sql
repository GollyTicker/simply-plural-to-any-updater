CREATE EXTENSION IF NOT EXISTS pgcrypto;
/*
secrets fields are stored as encrypted bytea fields.
we use the users hash(uuid + password) as the secret.
This secret is same for an individual users' fields,
but different for each user.
We also need to ensure, that when the user changes their password,
that then these secrets are re-encrypted.

how to insert:
    INSERT INTO users (username, password_hash, discord_token)
    VALUES (
        'testuser',
        'some_hash',
        pgp_sym_encrypt('your_discord_token', 'your_secret_key')
    );

how to access:
    SELECT
        username,
        pgp_sym_decrypt(discord_token, 'your_secret_key')::TEXT AS decrypted_discord_token
    FROM users
    WHERE username = 'testuser';
*/

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    email VARCHAR(127) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    wait_seconds INTEGER CHECK (wait_seconds > 0),
    request_timeout INTEGER CHECK (request_timeout > 0),
    
    system_name TEXT,
    status_prefix TEXT,
    status_no_fronts TEXT,
    status_truncate_names_to INTEGER CHECK (status_truncate_names_to > 0),
    
    enable_discord BOOLEAN NOT NULL DEFAULT false,
    enable_vrchat BOOLEAN NOT NULL DEFAULT false,
    
    /* encrypted secrets. need to be re-encrypted, when password changes. */
    enc__simply_plural_token bytea,
    enc__discord_token bytea,
    enc__vrchat_username bytea,
    enc__vrchat_password bytea,
    enc__vrchat_cookie bytea

    /* constraints to check manually before inserting into db:
    whenever a platform is enabled, the corresponding fields must be not null.*/
);
