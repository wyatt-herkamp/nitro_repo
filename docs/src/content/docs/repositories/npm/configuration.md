---
title: npm configuration
description: The settings on an npm repository.
sidebar:
  order: 4
---

An npm repository requires the `npm` config. `project` and `page` are optional and shared with
Maven.

## `npm`

```json
{ "type": "Hosted" }
```

`Hosted` is the only mode. There is no `Proxy` variant, so an npm repository cannot mirror
npmjs.org — configure the public registry alongside yours and scope your own packages:

```ini
@nitro:registry=https://repo.example.com/repositories/local/npm/
```

## `project`

Shared with Maven. Badge appearance and version validation.

| Field                        | Type                                | Default   |
| ---------------------------- | ----------------------------------- | --------- |
| `badge_settings.style`       | `flat` \| `plastic` \| `flatsquare` | `flat`    |
| `badge_settings.label_color` | colour                              | `#555`    |
| `badge_settings.color`       | colour                              | `#33B5E5` |
| `require_semver`             | boolean                             | `false`   |

npm versions are semver by definition, so `require_semver` is close to a no-op here. It is the same
config type Maven uses, which is why it appears.

A repository **without** a `project` config cannot serve badges — that is where their settings
live. See [Badges](/admin/badges/).

## `page`

Shared with Maven. The content shown on the repository's page in the UI.

| Field       | Type                           | Default |
| ----------- | ------------------------------ | ------- |
| `page_type` | `Markdown` \| `HTML` \| `None` | —       |
| `content`   | string or null                 | `null`  |

A good place for the `.npmrc` your team should be using.

## Repository-level settings

These are on the repository itself rather than in a config:

**Visibility.** `Public` installs need no credentials; `Private` requires a token with read access.
`Hidden` is readable but absent from search and listings. Writes always require authentication.

**Active.** An inactive repository refuses every request from everyone. Use it to retire a registry
without deleting what is in it.

## No push rules

There is no npm equivalent of `maven_push_rules`. The behaviour those would control is fixed:

- Republishing an existing version is always refused.
- Overwriting is never allowed.
- Any user with `Write` may publish any package name; there is no per-package ownership.
