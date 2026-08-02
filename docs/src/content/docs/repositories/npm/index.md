---
title: npm
description: A hosted npm registry — what is supported, and what is not there yet.
sidebar:
  order: 1
---

An npm repository is a hosted registry. Point `.npmrc` at it and `npm publish` / `npm install` work
as they do against any registry.

```text
{app_url}/repositories/{storage}/{repository}/
```

:::caution[The trailing slash is not optional]
Without it npm drops the last path segment when resolving relative URLs, and requests land under
`/repositories/{storage}/` instead. The symptom is a confusing 404 for a package that exists.
:::

## What works

|                         |                                                                             |
| ----------------------- | --------------------------------------------------------------------------- |
| **Publish and install** | `npm publish`, `npm install`, yarn and pnpm.                                |
| **Scoped packages**     | `@scope/name`, stored and served under the same key.                        |
| **dist-tags**           | `latest` and your own, with the full `dist-tag` command set.                |
| **Login**               | Both flows: the browser one npm 9+ defaults to, and the legacy CouchDB one. |
| **Unpublish**           | `npm unpublish`, revision-checked.                                          |
| **Deprecate**           | `npm deprecate`.                                                            |
| **Search**              | `npm search` against the registry.                                          |
| **Integrity**           | The tarball is checked against the `integrity` the client declared.         |
| **Access control**      | Reads honour repository visibility; writes require a `Write` action.        |

## What is not there

- **No proxying.** There is no `Proxy` mode for npm, so a registry cannot mirror npmjs.org. Keep
  the public registry configured alongside yours and scope your own to a prefix.
- **No `npm owner` or `npm access` semantics.** The commands are recognised rather than rejected,
  but there is no per-package ownership model behind them.
- **No `npm star`.**

## Package names

Nitro Repo enforces npm's real rules rather than a stricter approximation:

- At most 214 characters.
- Lowercase only. An uppercase character is refused with a message saying so.
- `a-z`, `0-9`, and `-`, `.`, `_`, `~`.
- No leading `.` or `_` — npm reserves those, and they would collide with path segments once the
  name becomes a storage path.
- A `/` only in a scoped name, and only one: `@scope/name`.

So `lodash.merge` publishes, and so does `@nitro/example`.

## The packument

A `GET` on a package name returns its packument — the document npm reads to decide what versions
exist and which one `latest` is.

```json
{
  "_id": "@nitro/example",
  "name": "@nitro/example",
  "dist-tags": { "latest": "1.1.0", "next": "2.0.0-beta.1" },
  "versions": {
    "1.0.0": {
      "name": "@nitro/example",
      "version": "1.0.0",
      "dist": { "tarball": "…", "integrity": "…", "shasum": "…" }
    }
  }
}
```

It is generated from what the registry holds, and versions come back in a defined order — which is
what makes `dist-tags.latest` deterministic. It was not always; without an explicit ordering,
"latest" was whichever row Postgres happened to return first.

## Next

- [Authenticating](/repositories/npm/authenticating/) — both login flows, and CI.
- [Publishing](/repositories/npm/publishing/) — publish, unpublish, deprecate, dist-tags.
- [Configuration](/repositories/npm/configuration/)
- [Troubleshooting](/repositories/npm/troubleshooting/)
