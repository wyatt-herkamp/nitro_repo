-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_auth_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
        CONSTRAINT fk_user_auth_tokens_user_id
            FOREIGN KEY (user_id)
                REFERENCES users (id)
                ON DELETE CASCADE,
    scopes JSONB,
    token TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS user_password_reset_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
        CONSTRAINT fk_user_password_reset_tokens_user_id
            FOREIGN KEY (user_id)
                REFERENCES users (id)
                ON DELETE CASCADE,
    request_details JSONB NOT NULL,
    state TEXT NOT NULL DEFAULT 'Requested',
    token TEXT NOT NULL,
        CONSTRAINT user_password_reset_tokens_token_unique UNIQUE (token),
    -- Once a token is used, It should not be changed again. So this value starts as NULL
    state_changed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_events(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
        CONSTRAINT fk_user_events_user_id
            FOREIGN KEY (user_id)
                REFERENCES users (id)
                ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    event_details JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
