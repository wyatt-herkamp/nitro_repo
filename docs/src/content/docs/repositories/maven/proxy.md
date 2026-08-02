---
title: Proxy repositories
description: Caching Maven Central and other upstreams — routes, priority, TTLs and credentials.
sidebar:
  order: 3
---

A proxy repository stores nothing of its own. It forwards a miss to its configured upstreams,
serves what comes back, and keeps a copy so the next request does not leave the building.

Point your builds at the proxy instead of at Central and you get the usual benefits: builds that do
not depend on someone else's uptime, a single place to see what your projects actually pull in, and
one egress path instead of one per developer.

## Setting one up

Create a Maven repository, then set its `maven` config to `Proxy` and add routes:

```json
{
  "type": "Proxy",
  "config": {
    "routes": [
      {
        "name": "central",
        "url": "https://repo1.maven.org/maven2",
        "priority": 1
      },
      {
        "name": "vendor",
        "url": "https://repo.vendor.example/maven",
        "priority": 2
      }
    ],
    "cache_ttl_seconds": 0,
    "mutable_ttl_seconds": 900
  }
}
```

## Routes

| Field      | Required | What                                                          |
| ---------- | -------- | ------------------------------------------------------------- |
| `url`      | Yes      | The upstream base URL, with no trailing slash.                |
| `name`     | No       | A label, for logs and the UI.                                 |
| `priority` | No       | Lower is tried first. A route with no priority is tried last. |
| `username` | No       | For an upstream that needs authentication.                    |
| `password` | No       | Password or token for that upstream.                          |

Routes are tried in priority order until one returns the file. A route that fails to connect is
logged and skipped rather than failing the request, so one unreachable upstream does not take the
proxy down.

Once a route has served a path, that route is tried **first** for related paths regardless of
priority. Walking every configured upstream on every request is the expensive part of a miss, and
the artifact you want is almost always where the last one came from.

### Credentials

`username` and `password` are stored in the repository's config, which means they are in Postgres
in plain text and visible to anyone who can read the repository's configuration. Use a
read-only credential from the upstream, not a general-purpose one.

## Caching

| Setting               | Default | What it covers                            |
| --------------------- | ------- | ----------------------------------------- |
| `cache_ttl_seconds`   | `0`     | Artifacts. `0` means keep forever.        |
| `mutable_ttl_seconds` | `900`   | `maven-metadata.xml` and snapshot builds. |

The split is the point. A released artifact is immutable by convention — `commons-lang3-3.14.0.jar`
is the same bytes forever — so the default of "cache it permanently" is right and re-fetching it
would be waste. Metadata is not: it changes every time anything is published upstream, so caching it
permanently means never seeing a new version. Fifteen minutes is a reasonable compromise between
freshness and hammering Central.

Raise `cache_ttl_seconds` above zero only if you are proxying somewhere that rewrites released
artifacts. Some internal repositories do; Central does not.

## Downloading a whole version

When a POM is fetched, the proxy also fetches the rest of that version — the jar, the sources jar,
the javadoc jar — in the background. Maven asks for the POM first and the jar immediately after, so
by the time the second request arrives the file is usually already local.

Files the upstream does not have are skipped quietly. A version with no sources jar is normal, not
a problem worth logging loudly.

`HEAD` is proxied as well as `GET`. Maven uses `HEAD` to decide whether a `GET` is worth making, so
a `HEAD` that only consulted the local cache would report "not there" for everything not yet
fetched and the `GET` would never happen.

## Indexing

Proxied artifacts are registered as projects and versions, exactly as deployed ones are, so a proxy
repository shows up in browse, search and badges.

This only covers what has actually been fetched. A proxy has no inventory of its upstream — it
discovers artifacts by being asked for them — so a freshly created proxy is empty to search and
fills in as builds pull through it. There is no "index Central" operation, and there could not
reasonably be one.

## Visibility

Proxy repositories obey visibility like any other. A private proxy requires authentication to read,
which is the usual arrangement when the proxy is for internal builds.

## What a proxy will not do

- **It will not accept a deploy.** A proxy is read-only; publish to a hosted repository.
- **It will not proxy npm.** There is no proxy mode for npm registries yet.
- **It will not group repositories.** Each proxy has its own upstream list; there is no repository
  that unions a hosted one and a proxy one into a single URL. Configure both in your build.
