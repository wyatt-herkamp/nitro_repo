-- Add up migration script here
CREATE TABLE IF NOT EXISTS storages (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE ignoreCase,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    config JSONB NOT NULL,
    created_at TIMESTAMP
    WITH
        TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP
    WITH
        TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS repositories (
    id UUID PRIMARY KEY,
    storage_id UUID NOT NULL,
    name TEXT NOT NULL UNIQUE COLLATE ignoreCase,
    repository_type TEXT NOT NULL,
    repository_subtype TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP
    WITH
        TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP
    WITH
        TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CONSTRAINT fk_repositories_storage_id FOREIGN KEY (storage_id) REFERENCES storages (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS repository_configs (
    id SERIAL PRIMARY KEY,
    repository_id UUID NOT NULL,
    key TEXT NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMP
    WITH
        TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP
    WITH
        TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CONSTRAINT fk_repository_configs_repository_id FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE
)