-- Distribution tags for npm projects.
--
-- Tags could not be stored at all before this, so `dist-tags` in a packument only ever contained
-- `latest`, synthesized from whichever version row came back first. `npm publish --tag next` and
-- `npm dist-tag add` had nowhere to write.
create TABLE IF NOT EXISTS npm_dist_tags
(
    project_id UUID                                               not null
        constraint fk_project
            references projects
            on delete cascade,
    tag        TEXT                                               not null,
    version    TEXT                                               not null,
    CONSTRAINT npm_dist_tags_pk PRIMARY KEY (project_id, tag),
    updated_at TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null,
    created_at TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP not null
);
