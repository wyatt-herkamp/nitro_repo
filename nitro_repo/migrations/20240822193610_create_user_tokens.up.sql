-- This migration creates the tables for user tokens
CREATE TABLE IF NOT EXISTS user_auth_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
        CONSTRAINT fk_user_auth_tokens_user_id
            FOREIGN KEY (user_id)
                REFERENCES users (id)
                ON DELETE CASCADE,
    token TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    source TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Scopes are used to define the permissions of the user_auth_token
CREATE TABLE IF NOT EXISTS user_auth_token_scopes(
    id SERIAL PRIMARY KEY,
    user_auth_token_id INTEGER NOT NULL,
        CONSTRAINT fk_user_auth_token_scopes_user_auth_token_id
            FOREIGN KEY (user_auth_token_id)
                REFERENCES user_auth_tokens (id)
                ON DELETE CASCADE,
    scope TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Repository Scopes are tied to a specific repository.
-- If the user has the regular scope for `ReadRepository`, they can read all repositories. and so on.
-- So this table is used to define the scopes for a specific repository. if the user has a scope for all repositories
CREATE TABLE IF NOT EXISTS user_auth_token_repository_scopes(
    id SERIAL PRIMARY KEY,
    user_auth_token_id INTEGER NOT NULL,
        CONSTRAINT fk_user_auth_token_repository_scopes_user_auth_token_id
            FOREIGN KEY (user_auth_token_id)
                REFERENCES user_auth_tokens (id)
                ON DELETE CASCADE,
    repository UUID NOT NULL,
        CONSTRAINT fk_user_auth_token_repository_scopes_repository
            FOREIGN KEY (repository)
                REFERENCES repositories (id)
                ON DELETE CASCADE,
    actions TEXT[] NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
)