-- Indexes on the `hostnames` foreign keys.
--
-- `hostnames` has been in the schema since the storages/repositories migration but was never used.
-- Custom domains make it load-bearing: listing a repository's hostnames and cascading a repository
-- delete both had to sequentially scan the table. `hostname` itself is already indexed by its
-- UNIQUE constraint, under the `ignoreCase` collation, which is what makes host lookups
-- case-insensitive.
CREATE INDEX IF NOT EXISTS hostnames_repository_id_idx ON hostnames (repository_id);

CREATE INDEX IF NOT EXISTS hostnames_storage_id_idx ON hostnames (storage_id);
