ALTER TABLE users
ADD COLUMN show_members_non_archived BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN show_members_archived BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN show_custom_fronts BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN respect_front_notifications_disabled BOOLEAN NOT NULL DEFAULT true;
