---
title: What is Nitro Repo?
description: An artifact manager for Maven and npm — what it does, what it does not do yet, and who it is for.
sidebar:
  order: 1
---

Nitro Repo is an artifact manager. It sits between the tools that produce build outputs — `mvn
deploy`, `npm publish` — and the tools that consume them, storing what was published and serving it
back with the metadata each ecosystem expects.

It is free, open source, and self-hosted. The backend is Rust; the frontend is Vue.

## What an artifact manager is for

A build produces artifacts: a jar, a tarball, a POM, a set of checksums. Something has to hold them
so the next build can find them. You could use a shared directory and a web server, and for one
project that works. It stops working as soon as you need any of the following:

- **Resolution metadata.** Maven asks for `maven-metadata.xml` to answer "what is the newest
  version", and for a per-version metadata file to resolve a `-SNAPSHOT` to an actual timestamped
  build. npm asks for a _packument_ listing every version and its dist-tags. Neither is a file your
  build produced; both have to be generated from what was published.
- **Access control.** Some repositories are public, some are internal, and the client sends
  credentials in an ecosystem-specific way.
- **Caching upstreams.** Every build pulling straight from Maven Central is slow, fragile, and
  invisible.
- **Knowing what is in there.** Searching, browsing, and seeing when a version was published.

## What Nitro Repo does today

|             |                                                                                                                                                                            |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Maven**   | Hosted and proxy repositories. Generated `maven-metadata.xml`, including timestamped snapshots. Checksums verified on upload and generated on demand. Push rules enforced. |
| **npm**     | A hosted registry. Scoped packages, dist-tags, publish, unpublish, deprecate, search, and both login flows — including the browser one npm 9 and later default to.         |
| **Storage** | The local filesystem, a self-contained single-file object format, or S3 and anything S3-compatible. Chosen per storage at runtime.                                         |
| **Search**  | A query language over projects and versions, exposed as both plain-text search and full query syntax.                                                                      |
| **Badges**  | Generated SVG badges for a repository or a project.                                                                                                                        |

Repository types are a trait, not a hardcoded list, so adding one is a matter of implementing it
rather than restructuring the server. NuGet, Cargo, Docker and apt/RPM are open issues, not shipped
features — [Compared with others](/about/comparison/) has where things actually stand.

## What it is not

- **Not a build tool.** It stores what your build produced; it does not produce it.
- **Not clustered.** One server, one Postgres. Storages can be S3, so the artifacts themselves can
  live anywhere, but there is no coordination between multiple Nitro Repo processes.
- **Not stable yet.** 2.0 is in beta and breaking changes are still on the table.

## Why it exists

The alternatives are either expensive, heavy, or both. Nexus is clunky; JFrog costs money at a
scale most people never reach. Nitro Repo aims at the case in between: a single team, a handful of
repositories, a machine that already exists, and no appetite for running a JVM the size of the
artifacts it serves.

## Next

- [Installation](/start/installation/) — Docker, a binary, or a build from source.
- [First run](/start/first-run/) — the administrator, a storage, a repository.
- [Core concepts](/start/concepts/) — how storages, repositories, projects and versions nest.
