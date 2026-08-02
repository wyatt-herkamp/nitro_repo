---
title: Cargo configuration
description: The settings on a Cargo repository.
sidebar:
  order: 4
---

A Cargo repository requires the `cargo` config. `project` and `page` are optional and shared with
Maven and npm.

## `cargo`

```json
{ "type": "Hosted" }
```

`Hosted` is the only mode. There is no `Proxy` variant, so a Cargo repository cannot mirror
crates.io — configure both registries side by side and name yours explicitly on the dependencies
that come from it:

```toml
[dependencies]
serde = "1"                                             # crates.io
nitro-example = { version = "1", registry = "nitro" }   # yours
```

## `config.json`

Not a stored config — the registry generates it per request, because the right answer depends on how
the client reached it:

```json
{
  "dl": "https://repo.example.com/repositories/local/crates/api/v1/crates",
  "api": "https://repo.example.com/repositories/local/crates",
  "auth-required": false
}
```

A request that arrived on a [custom domain](/admin/custom-domains/) is answered with that domain's
root instead. Telling a client to go back through `/repositories/{storage}/{name}` would work only
where both hosts resolve.

`auth-required` follows the repository's visibility: `true` for anything not `Public`. It is what
makes Cargo send a token on index requests as well as on publish.

:::caution[`app_url` must be set]
Without a custom domain, the registry has nothing but `site.app_url` to build those absolute URLs
from. With none configured, `config.json` fails rather than emitting a URL pointing nowhere. See
[Installation](/start/installation/#behind-a-reverse-proxy).
:::

## `project`

Shared with Maven and npm. Badge appearance and version validation.

| Field                        | Type                                | Default   |
| ---------------------------- | ----------------------------------- | --------- |
| `badge_settings.style`       | `flat` \| `plastic` \| `flatsquare` | `flat`    |
| `badge_settings.label_color` | colour                              | `#555`    |
| `badge_settings.color`       | colour                              | `#33B5E5` |
| `require_semver`             | boolean                             | `false`   |

Cargo versions are semver by definition, so `require_semver` is close to a no-op here.

A repository **without** a `project` config cannot serve badges — that is where their settings live.
See [Badges](/admin/badges/).

## `page`

Shared with Maven and npm. The content shown on the repository's page in the UI.

| Field       | Type                           | Default |
| ----------- | ------------------------------ | ------- |
| `page_type` | `Markdown` \| `HTML` \| `None` | —       |
| `content`   | string or null                 | `null`  |

A good place for the `.cargo/config.toml` your team should be using.

## Repository-level settings

**Visibility.** `Public` downloads need no credentials; `Private` requires a token with read access,
and sets `auth-required` so Cargo knows to send one. `Hidden` is readable but absent from search and
listings. Writes always require authentication.

**Active.** An inactive repository refuses every request from everyone.

## No push rules

There is no Cargo equivalent of `maven_push_rules`. The behaviour those would control is fixed:

- Re-publishing an existing version is always refused.
- Overwriting is never allowed.
- Any user with `Write` may publish any crate name. Ownership is recorded on first publish and
  governs `cargo owner`, but it does not reserve the name in advance.
