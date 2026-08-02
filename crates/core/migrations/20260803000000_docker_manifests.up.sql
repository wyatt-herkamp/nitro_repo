-- Manifests a Docker repository holds, addressed by digest.
--
-- Tags are NOT here: a tag is a version of a project, so it lives in `project_versions` alongside
-- every other repository type's versions, which is what gives Docker images the project pages,
-- badges and search everything else already has.
--
-- This table exists for the two things `project_versions` cannot express:
--   * a digest-addressed pull (`GET /v2/{name}/manifests/sha256:...`), which never names a tag;
--   * an untagged manifest — the per-platform children of an index, and OCI referrers artifacts —
--     which is reachable and must be retained even though no tag points at it.
CREATE TABLE IF NOT EXISTS docker_manifests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    repository_id UUID NOT NULL,
    image_name TEXT COLLATE ignoreCase NOT NULL,
    -- `algorithm:hex`, exactly as the client sent it. The digest is over the stored bytes, so this
    -- is the manifest's identity and not merely a checksum of it.
    digest TEXT NOT NULL,
    media_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    -- The OCI referrers relationship: this manifest is *about* the manifest named here.
    subject_digest TEXT,
    artifact_type TEXT,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_docker_manifests_repository FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE,
    CONSTRAINT docker_manifests_unique UNIQUE (repository_id, image_name, digest)
);

CREATE INDEX IF NOT EXISTS docker_manifests_image ON docker_manifests (repository_id, image_name);

-- `GET /v2/{name}/referrers/{digest}` filters on exactly this.
CREATE INDEX IF NOT EXISTS docker_manifests_subject ON docker_manifests (repository_id, image_name, subject_digest);
