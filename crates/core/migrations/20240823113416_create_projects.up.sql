create  TABLE IF NOT EXISTS  projects
(
    id                 UUID                     default gen_random_uuid() PRIMARY KEY,
    scope              TEXT,
    key                TEXT COLLATE ignoreCase                               not null,
    name               TEXT,
    description        TEXT,
    repository_id         UUID                                               not null
        constraint fk_repositories
            references repositories
            on delete cascade,
    CONSTRAINT unique_repository_project_key unique (key, repository_id),
    path       TEXT                                               not null,
    CONSTRAINT unique_repository_project_path  unique (path, repository_id),
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
    constraint unique_project_member unique (project_id, user_id),
    can_write  boolean,
    can_manage boolean,
    added_at   TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null
);

create TABLE IF NOT EXISTS  project_versions
(
    id                 UUID                     default gen_random_uuid() PRIMARY KEY,
    project_id   uuid
        constraint project_versions_projects_id_fk
            references projects
            on delete cascade,
    version      VARCHAR(255)                                               not null,
        CONSTRAINT unique_project_version unique (project_id, version),

    release_type VARCHAR(255)                     default 'Unknown'         not null,
    path TEXT                                                       not null,

    CONSTRAINT unique_version_path  unique (path, project_id),

    publisher    integer
        constraint project_versions_users_id_fk
            references users
            on delete set null,
    version_page TEXT,
    extra        JSONB                    default '{}'::JSONB       not null,
    updated_at   TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null,
    created_at   TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null
);