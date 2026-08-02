---
title: Seeding an instance
description: Fill an instance with a suite of Maven artifacts and npm packages over the real protocols.
sidebar:
  order: 6
---

An empty instance is hard to judge. Browsing, search, `maven-metadata.xml` merging, dist-tags and
badges all behave differently with one artifact than with forty, and most of the interesting
behaviour only appears once there is more than one version of something.

`nitro_repo seed` fills one. It creates the storages and repositories it needs and then deploys
through the **real Maven and npm protocols** — the same requests `mvn deploy` and `npm publish`
send — rather than writing rows into the database. A seed run is therefore also a smoke test of the
deploy paths: if seeding succeeds, publishing works.

## Running it

```sh
# write the bundled example, then edit it
nitro_repo seed --config seed.toml --write-example
$EDITOR seed.toml

nitro_repo seed --config seed.toml
```

From a checkout there is a recipe: `just seed seed.toml`.

At minimum, change the password and the storage path. The example points at
`http://localhost:6742` with the credentials `admin` / `changeme`.

**Re-running is safe.** Everything is declarative and idempotent — storages, repositories and
versions that already exist are left alone. Change the file and run it again.

## The file

```toml
url = "http://localhost:6742"

[auth]
username = "admin"
password = "changeme"
# or, preferred:
# token = "…"

[[storages]]
name = "local"
type = "Local"          # Local, FileSystemV2 or S3

[storages.settings]      # passed through to the backend untouched
path = "./storage/local"

[[repositories]]
storage = "local"
name = "releases"
type = "maven"           # maven or npm
visibility = "Public"

[[maven]]
repository = "local/releases"
group_id = "dev.kingtux"
artifact_id = "tms"
versions = ["1.0.0", "1.0.1", "1.1.0", "2.0.0-beta.1"]
description = "A seeded example artifact"
classifiers = ["sources", "javadoc"]
dependencies = ["org.slf4j:slf4j-api:2.0.13"]

[[npm]]
repository = "local/npm"
name = "@nitro/example"
versions = ["1.0.0", "1.0.1", "1.1.0"]
description = "A scoped package"

[npm.dist_tags]
next = "2.0.0-beta.1"
```

### Top level

| Key            | What                                            |
| -------------- | ----------------------------------------------- |
| `url`          | The base URL of the running instance.           |
| `auth`         | Either `{ token }` or `{ username, password }`. |
| `storages`     | Storages to create if missing.                  |
| `repositories` | Repositories to create if missing.              |
| `maven`        | Maven artifacts to deploy.                      |
| `npm`          | npm packages to publish.                        |

A token is preferable to a password — it is what a CI job would use, and it exercises the token
path. Basic auth is supported because it is what a freshly installed instance has: one admin
account and no tokens yet.

### `[[maven]]`

| Key            | Default                  | What                                                |
| -------------- | ------------------------ | --------------------------------------------------- |
| `repository`   | required                 | `{storage}/{repository}`.                           |
| `group_id`     | required                 | Also accepted as `group`.                           |
| `artifact_id`  | required                 | Also accepted as `artifact`.                        |
| `versions`     | required                 | Deployed in order.                                  |
| `description`  | none                     | Written into the generated POM.                     |
| `classifiers`  | `["sources", "javadoc"]` | Extra classified jars beside the main one.          |
| `dependencies` | `[]`                     | `groupId:artifactId:version`, written into the POM. |

A version ending in `-SNAPSHOT` is deployed as two timestamped builds, so `maven-metadata.xml` has
real `<snapshotVersions>` to merge rather than a single entry that would look correct either way.

### `[[npm]]`

| Key           | Default  | What                                                                              |
| ------------- | -------- | --------------------------------------------------------------------------------- |
| `repository`  | required | `{storage}/{repository}`.                                                         |
| `name`        | required | Including the scope: `@nitro/example`.                                            |
| `versions`    | required | Published in order.                                                               |
| `description` | none     | Written into `package.json`.                                                      |
| `dist_tags`   | `{}`     | Extra tags. `latest` is set to the last version published unless you set it here. |

## What the bundled example covers

The example is not arbitrary — it is built from the shapes that have broken before:

- **A scoped npm package** (`@nitro/example`), because scope handling was wrong for a long time in
  a way that only shows up when you publish one and then ask for it back.
- **A dotted unscoped name** (`nitro.helpers`), because dots were rejected outright, so
  `lodash.merge` could not have been published.
- **A prerelease** with a non-`latest` dist-tag.
- **A snapshot** version, so `maven-metadata.xml` has `<snapshotVersions>`.
- **A multi-version artifact**, so metadata has something to merge rather than one entry.
- **A nested group id**, to exercise deep paths.

## Seeding against a remote instance

`url` can point anywhere. Be careful — this publishes real artifacts under real coordinates, and
`dev.kingtux:tms` in your production `releases` repository is not something anyone wants to explain
later. Seed into a throwaway repository, or a throwaway instance.
