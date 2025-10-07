ALTER TABLE users
    ALTER COLUMN privacy_fine_grained SET DEFAULT 'NoFineGrained',
    ALTER COLUMN privacy_fine_grained_buckets SET DEFAULT '{}';