-- Add up migration script here
CREATE TABLE IF NOT EXISTS stages (
    id                 UUID                     default gen_random_uuid()   not null
        constraint stages_pk
            primary key,
    repository         UUID                                                 not null
        constraint fk_repositories
            references repositories
            on delete cascade,
    stage_state        JSONB                                                not null,
    created_by    integer                                                   not null
        constraint fk_user
            references users
            on delete cascade,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP          NOT NULL
);

CREATE TABLE IF NOT EXISTS stage_files (
    id                 UUID                     default gen_random_uuid()   not null
        constraint stage_file_pk
            primary key,
    stage              UUID                                                 not null
        constraint fk_stages
            references stages
            on delete cascade,
    file_name          TEXT                                                 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP          NOT NULL
);