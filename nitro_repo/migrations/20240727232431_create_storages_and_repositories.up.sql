-- Add up migration script here
CREATE TABLE IF NOT EXISTS storages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE COLLATE ignoreCase,
    storage_type TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    config JSONB NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    storage_id UUID NOT NULL,
    name TEXT NOT NULL COLLATE ignoreCase,
    repository_type TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_repositories_storage_id FOREIGN KEY (storage_id) REFERENCES storages (id) ON DELETE CASCADE,
    CONSTRAINT unique_repository_name UNIQUE (storage_id, name)
);

CREATE TABLE IF NOT EXISTS repository_configs (
    id SERIAL PRIMARY KEY,
    repository_id UUID NOT NULL,
    key TEXT NOT NULL,
    value JSONB NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_repository_configs_repository_id FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE,
    CONSTRAINT unique_repository_config_key UNIQUE (repository_id, key)
)