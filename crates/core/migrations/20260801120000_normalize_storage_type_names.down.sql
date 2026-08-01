-- Restores the lowercase spelling the S3 backend used to register itself under.
UPDATE storages SET storage_type = 's3' WHERE storage_type = 'S3';
