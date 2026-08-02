---
title: Repository types
description: What a repository type is, which ones exist, and how configs attach to them.
sidebar:
  order: 1
---

A repository's **type** decides how requests to it are interpreted. The same `PUT` of a file means
something specific to a Maven repository and nothing at all to an npm one.

Four types ship:

| Type     | Modes         | Clients                                                         |
| -------- | ------------- | --------------------------------------------------------------- |
| `maven`  | Hosted, Proxy | Maven, Gradle, sbt, anything else that speaks the Maven layout. |
| `npm`    | Hosted        | npm, yarn, pnpm.                                                |
| `cargo`  | Hosted        | `cargo`, over the sparse index.                                 |
| `docker` | Hosted        | Docker, podman, containerd, BuildKit, `crane`, `skopeo`.        |

The type is chosen at creation and cannot be changed afterwards — the layout, the metadata and the
authentication all differ, so there is nothing to convert.

## Hosted and proxy

A **hosted** repository stores what you publish to it. A **proxy** repository stores nothing of its
own: it forwards misses to configured upstreams and caches what comes back.

Only Maven has a proxy mode today. npm, Cargo and Docker repositories are always hosted; there is no
`Proxy` variant in their configs, so pointing one at npmjs.org, crates.io or Docker Hub is not
possible yet.

## How a repository is addressed

Three of the four live under one URL:

```text
{app_url}/repositories/{storage}/{repository}/
```

Docker is the exception, and not by choice: a Docker client always requests `/v2/...` at the root of
its host and cannot be given a path prefix. So a Docker repository is reached either on a
[custom domain](/admin/custom-domains/) of its own, or by putting `{storage}/{repository}` at the
front of the image name. See [Docker](/repositories/docker/#addressing).

Any repository can have a custom domain attached; for Docker it is the difference between
`docker.example.com/myimage` and `repo.example.com/local/docker/myimage`.

## Configs

Behaviour is set by **configs** — named JSON blobs validated against a schema the type publishes,
stored as separate rows so a type can gain settings without a migration.

| Config             | Maven | npm | Cargo | Docker | What it sets                                         |
| ------------------ | :---: | :-: | :---: | :----: | ---------------------------------------------------- |
| `maven`            |   ●   |     |       |        | Hosted or proxy, and the proxy's upstream routes.    |
| `maven_push_rules` |   ●   |     |       |        | Release/snapshot policy, overwrite, yanking, auth.   |
| `npm`              |       |  ●  |       |        | Registry mode.                                       |
| `cargo`            |       |     |   ●   |        | Registry mode.                                       |
| `docker`           |       |     |       |   ●    | Registry mode.                                       |
| `project`          |   ●   |  ●  |   ●   |   ●    | Badge appearance, and whether semver is required.    |
| `page`             |   ●   |  ●  |   ●   |   ●    | The Markdown or HTML shown on the repository's page. |

A type declares which configs it _requires_ — the one named after it — and the create form asks for
those. The rest are optional and can be added later from **Admin → Repositories → _name_**.

A config a repository does not have is not an error; the type falls back to the schema's defaults.
The exception worth knowing is `project`: without it, badges are refused, because that is where
their settings live.

## Adding a type

Repository types are a trait rather than a hardcoded list, so adding one means implementing the
trait and registering it — not restructuring the server. NuGet and apt/RPM are open issues, not work
in progress.

## Next

- [Maven](/repositories/maven/)
- [npm](/repositories/npm/)
- [Cargo](/repositories/cargo/)
- [Docker](/repositories/docker/)
