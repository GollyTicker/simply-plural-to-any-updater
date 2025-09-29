ALTER TABLE users ADD COLUMN enable_website BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN website_url_name TEXT;
ALTER TABLE users RENAME COLUMN system_name TO website_system_name;
