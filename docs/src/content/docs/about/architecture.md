---
title: Architecture
description: How the pieces fit — crates, request flow, what lives in Postgres, and what happens on a deploy.
sidebar:
  order: 1
---

Useful if you are debugging an instance, or about to work on one.

## The crates

| Crate             | What it is                                                                     |
| ----------------- | ------------------------------------------------------------------------------ |
| `nitro_repo/`     | The server: HTTP, repository types, authentication. `main.rs` is only the CLI. |
| `crates/core/`    | Shared types, the database entities, and the migrations.                       |
| `crates/storage/` | Storage backends — Local, FileSystemV2, S3.                                    |
| `crates/aql/`     | The query language behind search. Depends on nothing but serde and thiserror.  |
| `crates/macros/`  | Macros the other crates use.                                                   |
| `crates/nr-api/`  | A client for the API.                                                          |
| `site/`           | The Vue frontend.                                                              |

`nitro_repo` is a library with a thin binary on top, which is what lets the integration tests build
the real router in-process rather than starting a server and binding a port.

## Request flow

```text
Axum router
├── /repositories/{storage}/{repository}/{*path}
│     → resolve the repository by name
│     → dispatch on HTTP method to the repository type
│     → storage backend
├── /api/…            → JSON handlers → Postgres
├── /badge/…          → SVG generation
└── everything else   → the embedded frontend bundle
```

Repository types implement a trait with a handler per method, so `maven/hosted`, `maven/proxy` and
`npm/hosted` are three implementations behind one route rather than three routes.

## What lives where

**Postgres** holds everything that is configuration or index: users, permissions, tokens and their
scopes, storages and their settings, repositories and their configs, and the project and version
rows behind browse, search and badges.

**The storage backend** holds the artifacts, and their metadata — content type, timestamps, hashes.

**An embedded key-value database** holds browser sessions. They are short-lived, high-churn and
worthless after a restart if you would rather they were, so they do not belong in Postgres.

The split matters when you are planning backups: Postgres without the storage gives you an index
pointing at nothing, and the storage without Postgres gives you files nobody can find. Back up both,
at the same time.

## Storage abstraction

The `Storage` trait is keyed on `(repository: Uuid, location: StoragePath)`. Every backend
implements the same operations — save, open, delete, exists, list, and metadata — and a repository
never knows which one it is talking to.

Repositories are keyed by UUID rather than by name, so renaming a repository does not move a single
byte.

Paths are validated at construction: empty segments, `.`, `..` and NUL are rejected before a path
can be built, so a request cannot escape its repository's prefix by asking for
`../../other-repository/`.

## What happens on a Maven deploy

`mvn deploy` sends a `PUT` per file. For each one:

1. **Authenticate.** Basic auth, resolved to a user, checked for `Write` on this repository.
2. **Check the push rules.** Policy, semver, overwrite, project membership. A failure is a `400`
   naming the rule, before anything is written.
3. **If it is a checksum**, compare it against the artifact it names rather than storing it.
4. **If it is a POM**, parse it and check its coordinates against the deploy path.
5. **Write to storage.** Hashes are computed as the bytes go past.
6. **If it was a POM**, register the project and version — name, description and license come from
   it. A failure here fails the deploy.

`maven-metadata.xml` is not written by any of this. It is generated when requested, from the
database at the artifact level and from a directory listing at the snapshot level, so it cannot
drift and two clients deploying concurrently cannot overwrite each other's version lists.

## What happens on an npm publish

npm sends **one** request containing the manifest and the tarball as a base64 attachment.

1. Authenticate and check `Write`.
2. Parse the packument. Validate the package name.
3. Refuse if the version already exists — `409`.
4. Recompute the tarball's integrity and compare it against what the client declared.
5. Write the tarball, register the project and version, and set dist-tags.

The whole publish is one request, which is why an npm publish is atomic in a way a Maven deploy is
not.

## Proxy repositories

A miss walks the configured routes in priority order, with the route that last served a related
path tried first. A hit is streamed to the client and written to storage, and the rest of the
version — sources, javadoc — is fetched in the background, because Maven asks for the POM and then
immediately the jar.

Proxied artifacts are registered like deployed ones, so a proxy repository is visible to browse,
search and badges — but only for what has actually been fetched. A proxy has no inventory of its
upstream.

## Frontend

Vue 3, Vite, and a router whose route table is the source for two generated artifacts:

- **`routes.json`**, which the server reads to decide which paths get the SPA's `index.html`. A
  route missing from it is a hard 404 on refresh.
- **`src/types/api.d.ts`**, generated from the OpenAPI document.

Both are generated at build time rather than maintained by hand, because both had drifted when they
were not.

The built bundle is **compiled into the binary** as a zip, behind the `frontend` feature. One file
to deploy, and no way for the frontend and backend to be different versions.

## Observability

`tracing` throughout, with an OpenTelemetry exporter configured under `[log]` and off by default.
Spans carry the repository, project and version where those are known, so a slow deploy can be
attributed to a coordinate rather than to a URL.
