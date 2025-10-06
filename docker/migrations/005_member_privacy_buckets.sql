CREATE TYPE privacy_fine_grained_enum AS ENUM ('NoFineGrained', 'ViaFriend', 'ViaPrivacyBuckets');

ALTER TABLE users
    ADD COLUMN privacy_fine_grained privacy_fine_grained_enum,
    ADD COLUMN privacy_fine_grained_buckets TEXT[];