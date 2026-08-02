---
title: Maven configuration
description: Every setting on a Maven repository, its default, and what it rejects.
sidebar:
  order: 4
---

Settings live in configs on the repository, editable from **Admin → Repositories → _name_**. A
Maven repository requires the `maven` config; the rest are optional.

## `maven`

Which mode the repository is in.

```json
{ "type": "Hosted" }
```

```json
{
  "type": "Proxy",
  "config": {
    "routes": [
      {
        "url": "https://repo1.maven.org/maven2",
        "name": "central",
        "priority": 1
      }
    ],
    "cache_ttl_seconds": 0,
    "mutable_ttl_seconds": 900
  }
}
```

`config` is required for `Proxy` and unused for `Hosted`. See
[Proxy repositories](/repositories/maven/proxy/).

## `maven_push_rules`

Applies to every deploy. All fields have defaults, so the config can be added with `{}` and
adjusted from there.

| Field                          | Type                               | Default |
| ------------------------------ | ---------------------------------- | ------- |
| `push_policy`                  | `Release` \| `Snapshot` \| `Mixed` | `Mixed` |
| `yanking_allowed`              | boolean                            | `true`  |
| `allow_overwrite`              | boolean                            | `true`  |
| `must_be_project_member`       | boolean                            | `false` |
| `must_use_auth_token_for_push` | boolean                            | `false` |

### `push_policy`

What version shapes the repository accepts.

- **`Release`** — refuses anything ending in `-SNAPSHOT`.
- **`Snapshot`** — refuses anything that is not a snapshot.
- **`Mixed`** — accepts both.

The conventional setup is two repositories, `releases` on `Release` and `snapshots` on `Snapshot`,
so a mis-pointed `distributionManagement` is caught at deploy time rather than discovered later.

A path with no version directory — `maven-metadata.xml` at the artifact level — is not judged.

### `allow_overwrite`

Off, a deploy to a path that already has a file is refused.

Files Maven legitimately rewrites are exempt: metadata documents and checksums. Without that
exemption, turning this off would break normal deploys rather than protecting anything.

Turning this off on a **release** repository is the right default for anything anyone else depends
on: a released version whose contents changed is a genuinely hard problem to debug. It is wrong for
a snapshot repository, where re-publishing is the point.

### `must_be_project_member`

On, only members of an existing project may push to it.

This is only meaningful once the project exists — the first push is what creates it, and whoever
made it becomes an owner. It is also weaker than it looks with plain `mvn deploy`, since Maven
uploads each file as a separate request with no notion of a deploy as a unit.

### `must_use_auth_token_for_push`

On, deploys must authenticate with an API token rather than a password. Put the token in the
password field of `settings.xml`; Basic auth is all Maven speaks either way.

Worth turning on. It means a leaked `settings.xml` costs you a scoped token rather than an account.

### `yanking_allowed`

Whether versions may be deleted from this repository.

## `project`

Shared with npm. Badge appearance and version validation.

| Field                        | Type                                | Default   |
| ---------------------------- | ----------------------------------- | --------- |
| `badge_settings.style`       | `flat` \| `plastic` \| `flatsquare` | `flat`    |
| `badge_settings.label_color` | colour                              | `#555`    |
| `badge_settings.color`       | colour                              | `#33B5E5` |
| `require_semver`             | boolean                             | `false`   |

`require_semver` refuses a deploy whose version is not valid semver. `1.0.0-SNAPSHOT` is a legal
semver prerelease, so snapshots pass; `1.0` and `2024.06` do not.

A repository **without** a `project` config cannot serve badges — that is where their settings live.
See [Badges](/admin/badges/).

## `page`

Shared with npm. The content shown on the repository's page in the UI.

| Field       | Type                           | Default |
| ----------- | ------------------------------ | ------- |
| `page_type` | `Markdown` \| `HTML` \| `None` | —       |
| `content`   | string or null                 | `null`  |

Use it for the things a README would say: what belongs in this repository, who to ask, what the
retention policy is.

## Rejections you might see

Every push-rule failure is a `400` naming the rule:

| Message                                                           | Cause                                           |
| ----------------------------------------------------------------- | ----------------------------------------------- |
| `This repository does not accept snapshot versions`               | `push_policy = Release`.                        |
| `This repository only accepts snapshot versions`                  | `push_policy = Snapshot`.                       |
| `… already exists and this repository does not allow overwriting` | `allow_overwrite = false`.                      |
| `You must be a member of this project to push to it`              | `must_be_project_member = true`.                |
| `… is not a valid semver version, which this repository requires` | `require_semver = true`.                        |
| `The POM describes … but was deployed to …`                       | Coordinates do not match the path.              |
| `Uploaded checksum does not match the stored artifact`            | A corrupted transfer, or a stale checksum file. |

## Removed: `require_nitro_deploy`

This flag used to exist and has been removed. Nitro Deploy was never implemented, so setting it
made a repository undeployable — every push got a `400`, with no client configuration that would
work. Config rows that still carry the field ignore it.
