UPDATE users
SET
    privacy_fine_grained = 'NoFineGrained',
    privacy_fine_grained_buckets = '{}'
WHERE
    privacy_fine_grained IS NULL;
