-- Normalise `storages.storage_type` to match the storage factory names.
--
-- The column stores the name a storage is looked up by, matched exactly and case-sensitively
-- against `StorageFactory::storage_name()`. The S3 backend used to report `"s3"` there while the
-- config JSON stored alongside it is tagged `"S3"` (the `StorageTypeConfig` variant name), so one
-- row carried two spellings of the same type. Both names are now `"S3"`.
--
-- Any row that does not match a factory is skipped at boot with a warning, so an un-migrated `s3`
-- row would silently stop loading its storage.
UPDATE storages SET storage_type = 'S3' WHERE storage_type = 's3';
UPDATE storages SET storage_type = 'Local' WHERE storage_type = 'local';
