---
title: Repository types
description: What a repository type is, which ones exist, and how configs attach to them.
sidebar:
  order: 1
---

A repository's **type** decides how requests to it are interpreted. The same `PUT` of a file means
something specific to a Maven repository and nothing at all to an npm one.

Two types ship:

| Type    | Modes         | Clients                                                         |
| ------- | ------------- | --------------------------------------------------------------- |
| `maven` | Hosted, Proxy | Maven, Gradle, sbt, anything else that speaks the Maven layout. |
| `npm`   | Hosted        | npm, yarn, pnpm.                                                |

The type is chosen at creation and cannot be changed afterwards — the layout, the metadata and the
authentication all differ, so there is nothing to convert.

## Hosted and proxy

A **hosted** repository stores what you publish to it. A **proxy** repository stores nothing of its
own: it forwards misses to configured upstreams and caches what comes back.

Only Maven has a proxy mode today. An npm repository is always hosted; there is no `Proxy` variant
in its config, so pointing one at the public registry is not possible yet.

## Configs

Behaviour is set by **configs** — named JSON blobs validated against a schema the type publishes,
stored as separate rows so a type can gain settings without a migration.

| Config             | Maven | npm | What it sets                                         |
| ------------------ | :---: | :-: | ---------------------------------------------------- |
| `maven`            |   ●   |     | Hosted or proxy, and the proxy's upstream routes.    |
| `maven_push_rules` |   ●   |     | Release/snapshot policy, overwrite, yanking, auth.   |
| `npm`              |       |  ●  | Registry mode.                                       |
| `project`          |   ●   |  ●  | Badge appearance, and whether semver is required.    |
| `page`             |   ●   |  ●  | The Markdown or HTML shown on the repository's page. |

A type declares which configs it _requires_ — `maven` and `npm` respectively — and the create form
asks for those. The rest are optional and can be added later from **Admin → Repositories → _name_**.

A config a repository does not have is not an error; the type falls back to the schema's defaults.
The exception worth knowing is `project`: without it, badges are refused, because that is where
their settings live.

## Adding a type

Repository types are a trait rather than a hardcoded list, so adding one means implementing the
trait and registering it — not restructuring the server. NuGet, Cargo, Docker and apt/RPM are open
issues, not work in progress.

## Next

- [Maven](/repositories/maven/)
- [npm](/repositories/npm/)
