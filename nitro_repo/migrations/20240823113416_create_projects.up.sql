-- Add up migration script here
-- Add up migration script here
create  TABLE IF NOT EXISTS  projects
(
    id                 UUID                     default gen_random_uuid() not null
        constraint projects_pk
            primary key,
    scope              TEXT,
    project_key        TEXT                                               not null,
    name               TEXT,
    latest_release     TEXT,
    latest_pre_release TEXT,
    description        TEXT,
    tags               TEXT[]                   default array []::text[]  not null,
    repository_id         UUID                                               not null
        constraint fk_repositories
            references repositories
            on delete cascade,
    storage_path       TEXT                                               not null,
    updated_at         TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null,
    created_at         TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null
);
create TABLE IF NOT EXISTS  project_members
(
    id  serial
        constraint project_members_pk
            primary key,
    project_id UUID                                               not null
        constraint fk_project
            references projects
            on delete cascade,
    user_id    integer                                            not null
        constraint fk_user
            references users
            on delete cascade,
    can_write  boolean,
    can_manage boolean,
    added_at   TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null
);

create TABLE IF NOT EXISTS  project_versions
(
    id           serial
        constraint project_versions_pk
            primary key,
    project_id   uuid
        constraint project_versions_projects_id_fk
            references projects
            on delete cascade,
    version      TEXT                                               not null,
    release_type TEXT                     default 'Unknown'         not null,
    version_path TEXT                                               not null,
    publisher    integer
        constraint project_versions_users_id_fk
            references users
            on delete set null,
    version_page TEXT,
    extra        JSONB                    default '{}'::JSONB       not null,
    updated_at   TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null,
    created_at   TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null
);

