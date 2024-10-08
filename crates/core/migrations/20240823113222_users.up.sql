-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    username TEXT NOT NULL UNIQUE COLLATE ignoreCase,
    email TEXT NOT NULL UNIQUE COLLATE ignoreCase,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    password TEXT,
    require_password_change BOOLEAN DEFAULT FALSE,
    admin BOOLEAN NOT NULL DEFAULT FALSE,
    user_manager BOOLEAN NOT NULL DEFAULT FALSE,
    system_manager BOOLEAN NOT NULL DEFAULT FALSE,
    default_repository_actions TEXT[] DEFAULT ARRAY []::text[]  not null,
    password_last_changed TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS user_repository_permissions(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
        CONSTRAINT fk_user_repository_permissions_user_id
            FOREIGN KEY (user_id)
                REFERENCES users (id)
                ON DELETE CASCADE,
    repository_id UUID NOT NULL,
        CONSTRAINT fk_user_repository_permissions_repository
            FOREIGN KEY (repository_id)
                REFERENCES repositories (id)
                ON DELETE CASCADE,
    CONSTRAINT unique_repository_and_user UNIQUE (user_id, repository_id),
    actions TEXT[] NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);