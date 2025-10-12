ALTER TABLE users ADD COLUMN enc__pluralkit_token bytea;
ALTER TABLE users ADD COLUMN enable_to_pluralkit BOOLEAN NOT NULL DEFAULT false;
