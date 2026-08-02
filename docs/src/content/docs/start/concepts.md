---
title: Core concepts
description: Storages, repositories, projects, versions, configs and scopes — what each one is and how they nest.
sidebar:
  order: 4
---

Six nouns explain most of Nitro Repo. They nest, and the nesting is what the URLs are made of.

```text
storage                       where bytes live
└── repository                what a client talks to
    └── project               a coordinate: dev.kingtux:tms, @nitro/example
        └── version           1.4.2, 1.5.0-SNAPSHOT
            └── file          the jar, the POM, the tarball, the checksums
```

## Storage

A **storage** is a place bytes go, and nothing more. It has a name, a type, and the configuration
that type needs. Three types ship:

- **Local** — ordinary files and directories on disk, with metadata in `.nr-meta` sidecar files
  alongside them.
- **FileSystemV2** — also on disk, but each file is a single self-contained object holding its own
  metadata, so there are no sidecars to drift and range requests work.
- **S3** — an S3 bucket, or anything speaking the S3 API. MinIO, Ceph, R2, Backblaze.

Storages are created at runtime and their configuration lives in Postgres, so adding one does not
mean restarting or rebuilding. Repositories are permanently assigned to the storage they were
created in; there is no move.

See [Storages](/admin/storages/).

## Repository

A **repository** is what a client talks to. It belongs to a storage, has a name, a type — `maven`
or `npm` — and a visibility. Its URL is the storage name and its own:

```text
{app_url}/repositories/{storage}/{repository}/
```

The type decides how requests are interpreted. The same `PUT` of a `.jar` means something specific
to a Maven repository and nothing at all to an npm one. Types are a trait rather than a fixed list,
so the two that exist today are not a ceiling.

A repository can also be **inactive**, which is separate from visibility: an inactive repository
refuses every request, including from people who would otherwise have access. Use it to retire
something without deleting what is in it.

## Project and version

A **project** is one coordinate within a repository. For Maven that is `groupId:artifactId`, split
into a _scope_ (`dev.kingtux`) and a _name_ (`tms`); for npm it is the package name, with the scope
being the `@nitro` in `@nitro/example`.

A **version** belongs to a project and is a single published release: `1.4.2`, `1.5.0-SNAPSHOT`,
`2.0.0-beta.1`. Each carries a release type — `Stable`, `Snapshot`, `Alpha`, `Beta`,
`ReleaseCandidate` or `Unknown` — inferred from the version string.

Projects and versions are **derived**, not declared. Nothing creates them directly; they appear
because something was deployed. For Maven that means a POM being uploaded, which is where the name,
description and license come from. This is also why they are what search, badges and the browse UI
read: the files are the truth, and these rows are the index over them.

## Config

A repository's behaviour is set by **configs**, each a named blob of JSON validated against a
schema the repository type publishes. They are separate rows rather than columns, so a repository
type can add settings without a migration.

| Config             | Applies to    | What it sets                                         |
| ------------------ | ------------- | ---------------------------------------------------- |
| `maven`            | Maven         | Hosted or proxy, and the proxy's upstream routes.    |
| `maven_push_rules` | Maven         | Release/snapshot policy, overwrite, yanking, auth.   |
| `npm`              | npm           | Registry mode.                                       |
| `project`          | Maven and npm | Badge appearance, and whether semver is required.    |
| `page`             | Maven and npm | The Markdown or HTML shown on the repository's page. |

A repository type declares which configs it requires; the create form asks for those, and the rest
can be added later from the repository's page.

## User, permission and scope

A **user** signs in and has permissions: `admin`, `user_manager`, `system_manager`, and a set of
default repository actions. Permissions can also be granted per repository, so read on one and
write on another is expressible.

An **API token** belongs to a user and carries **scopes**. The distinction that matters:

> A scope is a ceiling, never a grant.

Holding `DeleteRepository` on a token does not let you delete anything unless the user behind the
token could already. A token can only ever do less than its owner. That is why a CI token minted
with just `ReadRepository` and `WriteRepository` is safe to put in a pipeline: even if it leaks, it
cannot create users or rewrite settings, regardless of who owns it.

See [Users and permissions](/admin/users/) and [API tokens and scopes](/admin/tokens/).

## Where things are served from

| Path                                     | What                                                   |
| ---------------------------------------- | ------------------------------------------------------ |
| `/repositories/{storage}/{repository}/…` | The repository itself. This is what Maven and npm use. |
| `/api/…`                                 | The JSON API the frontend uses.                        |
| `/badge/{storage}/{repository}/…`        | Generated SVG badges.                                  |
| `/open-api-doc-raw`                      | The OpenAPI document, when `open_api_routes` is on.    |
| `/scalar`                                | Interactive API docs, same condition.                  |
| everything else                          | The frontend.                                          |
