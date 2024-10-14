-- Add up migration script here
CREATE TABLE IF NOT EXISTS repository_hostnames (
    id SERIAL PRIMARY KEY,
    repository_id         UUID                                              NOT NULL
        constraint fk_repositories_hostnames
            references repositories
            on delete cascade,
    hostname           TEXT UNIQUE COLLATE ignoreCase                     NOT NULL,
    updated_at         TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null,
    created_at         TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null
);
