---
title: Badges
description: Generated SVG version badges for a repository or a project, and how to style them.
sidebar:
  order: 5
---

Nitro Repo generates its own SVG badges — no shields.io, no external request, and they work for a
private instance that the internet cannot reach.

## The URLs

```text
GET /badge/{storage}/{repository}
GET /badge/{storage}/{repository}/project/{project}
GET /badge/{storage}/{repository}/supports
```

The first shows the repository's name. The second shows the latest version of one project, which is
the one you usually want in a README:

```markdown
![](https://repo.example.com/badge/local/releases/project/dev.kingtux:tms)
```

`{project}` is the project key: `dev.kingtux:tms` for Maven, `@nitro/example` for npm.

`/supports` is a probe. It answers `204` if the repository can produce badges and `400` if it
cannot, so a tool can decide whether to offer them without parsing an SVG.

## Which version is shown

By default, the latest **stable** version. Pass `release_type` to change that:

```text
/badge/local/snapshots/project/dev.kingtux:tms?release_type=Snapshot
```

It may be repeated to accept more than one, in which case the newest of any of them wins:

```text
?release_type=Stable&release_type=Beta
```

Values are `Stable`, `Snapshot`, `Alpha`, `Beta`, `ReleaseCandidate` and `Unknown`. An empty
`release_type` falls back to `Stable` rather than matching nothing.

## Appearance

Badge style lives in the repository's `project` config — **Admin → Repositories → _name_ →
Project**. The page has a colour picker, a live preview, and ready-made Markdown, HTML and
reStructuredText snippets to copy.

| Setting       | Default   | What                                |
| ------------- | --------- | ----------------------------------- |
| `style`       | `flat`    | `flat`, `plastic` or `flatsquare`.  |
| `label_color` | `#555`    | The left half, behind the label.    |
| `color`       | `#33B5E5` | The right half, behind the version. |

Settings are per repository, not per project, so every badge from one repository looks the same.

## If a badge does not render

**`This Repository does not support badges`** — the repository has no `project` config. Add it from
the repository's page; it is what holds the badge settings, and a repository created without it has
nowhere to read them from.

**`Project not found`** — the project key does not match. It is the full key including the scope,
and it is case-sensitive in the way the ecosystem is: `dev.kingtux:tms`, not `tms`.

**A badge that renders but shows nothing useful** — the project has no version of the requested
release type. A repository holding only `1.0.0-SNAPSHOT` has no stable version, so the default
badge has nothing to show. Pass `release_type=Snapshot`.

## Badges ignore visibility

:::caution[A badge from a private repository is still public]
The badge routes do not check authentication or repository visibility. Anyone who can reach the
instance can request a badge for any project in any repository and read back its latest version
number, whether the repository is `Public`, `Hidden` or `Private`.

The artifacts themselves stay protected — this leaks version numbers and project keys, not
contents. It is still more than a private repository should say to an anonymous caller, so treat
badge URLs as public information until this is tightened.
:::
