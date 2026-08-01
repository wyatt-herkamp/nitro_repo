-- `project_versions.release_type` was created as VARCHAR(255) while `ReleaseType` declares
-- `#[sqlx(type_name = "TEXT")]`. sqlx checks the column's type OID when decoding, so every read of
-- a row carrying this column failed with:
--
--     mismatched types; Rust type `ReleaseType` (as SQL type `TEXT`) is not compatible with
--     SQL type `VARCHAR`
--
-- which is why nothing that lists Maven versions ever returned any. It went unnoticed because
-- `post_pom_upload` only *logged* its errors until Phase 4 made them returned, so a deploy reported
-- 201 while its version registration had failed.
--
-- Every other enum-backed column in this schema is already TEXT; this brings the last one in line.
-- In Postgres the two are the same storage — varchar(n) is text plus a length check — so no data
-- moves and nothing is truncated.
ALTER TABLE project_versions
    ALTER COLUMN release_type TYPE TEXT;
