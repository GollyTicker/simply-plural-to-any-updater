DELETE FROM users WHERE email = '';
ALTER TABLE users ADD CONSTRAINT email_not_empty CHECK (email <> '');
