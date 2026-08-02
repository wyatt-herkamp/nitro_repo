---
title: Docker configuration
description: The settings on a Docker repository, and what an operator has to plan for.
sidebar:
  order: 4
---

A Docker repository requires the `docker` config. `project` and `page` are optional and shared with
the other types.

## `docker`

```json
{ "type": "Hosted" }
```

`Hosted` is the only mode. There is no `Proxy` variant, so a Docker repository cannot act as a
pull-through cache for Docker Hub.

## Hostnames

Not a config — [custom domains](/admin/custom-domains/) are attached to the repository itself, and
for Docker they are the difference between

```sh
docker pull docker.example.com/myimage:1.0
docker pull repo.example.com/local/docker/myimage:1.0
```

Both work at the same time. Adding a hostname does not break anyone using the prefixed form.

A hostname pointing at a repository that is _not_ a Docker one is unaffected: a `/v2/...` request on
such a host is served as an ordinary artifact path, so a Maven artifact stored under `v2/` keeps
working.

## Staging

An in-progress blob upload is buffered to the instance's staging directory before it is handed to
storage, because there is no streaming write on the storage backends.

```toml
[staging]
staging_dir = "./staging"
```

Two consequences worth planning for:

- **Disk.** A push needs room for the largest layer in flight, per concurrent upload, on the node's
  local disk — not in S3.
- **Affinity.** An upload session lives in one process's memory. Behind a load balancer, either pin
  a client's requests to one backend or run a single instance. A `PATCH` that lands on a different
  node answers `404 BLOB_UPLOAD_UNKNOWN`.

Abandoned sessions are swept after an hour.

## Storage growth

Nothing garbage-collects blobs. Deleting a tag or a manifest leaves its layers on disk, because
layers are shared between images and nothing counts references to them.

In practice this means a repository that CI pushes to on every commit grows without bound. Until
there is a GC, the options are to push fewer distinct layers, or to rebuild the repository
periodically from what you want to keep.

## `project`

Shared with the other types. Badge appearance and version validation.

| Field                        | Type                                | Default   |
| ---------------------------- | ----------------------------------- | --------- |
| `badge_settings.style`       | `flat` \| `plastic` \| `flatsquare` | `flat`    |
| `badge_settings.label_color` | colour                              | `#555`    |
| `badge_settings.color`       | colour                              | `#33B5E5` |
| `require_semver`             | boolean                             | `false`   |

Leave `require_semver` off. A container tag is not a version — `latest`, `edge` and `main` are all
legitimate — and turning it on would reject them.

A repository **without** a `project` config cannot serve badges. See [Badges](/admin/badges/).

## `page`

Shared with the other types. The content shown on the repository's page in the UI.

| Field       | Type                           | Default |
| ----------- | ------------------------------ | ------- |
| `page_type` | `Markdown` \| `HTML` \| `None` | —       |
| `content`   | string or null                 | `null`  |

## Repository-level settings

**Visibility.** `Public` pulls need no credentials at all. `Private` challenges every read, which is
what makes a client fetch a token rather than give up. `Hidden` is readable but absent from search
and listings. Writes always require authentication.

**Active.** An inactive repository refuses every request from everyone.

## Upload size

A layer has to fit inside the server's body limit:

```toml
[web_server]
max_upload = "2GiB"   # or "unlimited"
```

The default is 100 MiB, which is smaller than plenty of real base images. Raise it before the first
push of anything substantial.
