---
title: Maven
description: Hosted and proxy Maven repositories — what works, and how the pieces fit together.
sidebar:
  order: 1
---

A Maven repository serves the standard Maven layout, so anything that speaks it works: Maven,
Gradle, sbt, Leiningen, Mill.

```text
{app_url}/repositories/{storage}/{repository}/
```

## Two modes

**Hosted** stores what you deploy to it. This is your `releases` and your `snapshots`.

**Proxy** stores nothing of its own. It forwards misses to configured upstreams — Maven Central,
your vendor's repository — and caches what comes back, so the second build is fast and an upstream
outage does not stop you.

The mode is set by the `maven` config, and switching between them is possible after creation,
though a hosted repository holding artifacts should not become a proxy.

## What works

|                          |                                                                                                                                 |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------- |
| **Deploy**               | `mvn deploy`, Gradle's `maven-publish`, or any plain `PUT`.                                                                     |
| **Resolve**              | `GET`, with a directory index for a browser.                                                                                    |
| **`maven-metadata.xml`** | Generated from what the repository holds, at both the artifact and the snapshot level. Never taken from what a client uploaded. |
| **Snapshots**            | Timestamped deploys with per-version `<snapshotVersions>`, so resolving `1.0.0-SNAPSHOT` works.                                 |
| **Checksums**            | `.md5`, `.sha1`, `.sha256`, `.sha512` — verified on upload, generated on demand.                                                |
| **Push rules**           | Release/snapshot policy, overwrite, project membership, token-only pushes.                                                      |
| **POM consistency**      | A POM's coordinates must match the path it was deployed to.                                                                     |
| **Proxying**             | Multiple upstreams, priority-ordered, with TTLs and optional credentials.                                                       |

## The two metadata documents

Maven's metadata is the part people are surprised by, so it is worth naming the two documents up
front. Both are **generated**, never stored:

```text
{group}/{artifact}/maven-metadata.xml
```

Every version of an artifact. Maven reads it to resolve version ranges, `LATEST` and `RELEASE`.
It is built from the database, so two machines deploying the same artifact concurrently cannot
overwrite each other's version lists.

```text
{group}/{artifact}/{version}-SNAPSHOT/maven-metadata.xml
```

The timestamped builds inside one snapshot version. Maven reads it to turn a bare `1.0.0-SNAPSHOT`
into the actual file it should download. Without it, snapshot resolution does not work at all.
It is built by listing the directory, because individual snapshot builds are files rather than
database rows.

If a client uploads either document, it is ignored rather than stored. Client-supplied metadata is
a snapshot of what one machine knew at one moment, which is precisely wrong when more than one
machine deploys.

## Nitro Deploy

Older documentation described a custom multi-step deploy protocol — `POST /deploy`, files uploaded
individually against a deploy ID, then a publish step — behind a `require_nitro_deploy` flag.

**It was never implemented, and the flag has been removed.** Its types were referenced by nothing,
the handler was unimplemented, and a `PUT` carrying its header returned `405`. Setting the flag made
a repository undeployable: every push got a `400`, with no client configuration that would work.
Config rows that still carry the field simply ignore it.

Standard Maven deploy is the way to deploy.

## Next

- [Deploying and resolving](/repositories/maven/deploying/) — client setup, snapshots, checksums.
- [Proxy repositories](/repositories/maven/proxy/) — upstreams, caching, credentials.
- [Configuration](/repositories/maven/configuration/) — every setting, and what it rejects.
