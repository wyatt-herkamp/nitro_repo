---
title: npm troubleshooting
description: The errors people actually hit, and what causes each one.
sidebar:
  order: 5
---

## 404 for a package you know exists

**The registry URL is missing its trailing slash.**

```ini
registry=https://repo.example.com/repositories/local/npm    # wrong
registry=https://repo.example.com/repositories/local/npm/   # right
```

npm resolves package paths relative to the registry URL. Without the slash the last segment is
treated as a file name and dropped, so requests go to `/repositories/local/` and nothing is there.

This is the single most common problem, and the error npm reports gives no hint of it.

## 401 with a token that looks correct

**The `_authToken` key does not match the registry URL.** npm matches them by exact string, minus
the protocol:

```ini
registry=https://repo.example.com/repositories/local/npm/
//repo.example.com/repositories/local/npm/:_authToken=${NITRO_TOKEN}
```

The path must be identical, including the trailing slash. A mismatch is silent — npm simply does
not attach the token, and the registry sees an anonymous request.

Check what npm thinks it is doing:

```sh
npm config get registry
npm whoami --registry=$(npm config get registry)
```

## `npm login` fails or hangs

**503, "no `app_url` configured".** The browser flow needs to hand out absolute URLs and has no way
to build one. Set `site.app_url`, or use `npm login --auth-type=legacy`.

**The browser opens but the CLI never finishes.** Sessions last 15 minutes and are held in memory —
a server restart drops them. Run `npm login` again.

**The login URL is unreachable from your browser.** `app_url` is set to something only the server
can resolve. It should be the URL your users type.

## Invalid package name

The rules are npm's, and the message says which one was broken:

| Message                                     | Fix                                    |
| ------------------------------------------- | -------------------------------------- |
| `Name cannot contain uppercase characters`  | npm names are lowercase.               |
| `Name cannot start with a `.`or`_``         | Both are reserved by npm.              |
| `Name may only contain a-z, 0-9, and …`     | Allowed extras are `-`, `.`, `_`, `~`. |
| `Name is longer than 214 characters`        | npm's limit.                           |
| `Invalid scope format. Must be @scope/name` | One `/`, and only in a scoped name.    |

## 409 on publish

The version already exists. Published versions are immutable — that is the point of a registry.
Bump the version, or `npm unpublish` it first if it was a mistake and nothing has consumed it.

## Integrity mismatch on publish

The tarball does not match the `integrity` npm sent with it, which means the upload was corrupted
in transit. Retry. If it happens repeatedly, look at whatever is between npm and the server — a
proxy rewriting request bodies will do this.

## Publish fails with a large package

The upload exceeded `web_server.max_upload`, 100 MiB by default. Raise it, or raise the limit on
whatever proxy is in front — nginx's `client_max_body_size` defaults to 1 MiB and will reject the
request before Nitro Repo sees it.

## `npm access` / `npm owner` / `npm star` refused

Recognised, but not implemented. There is no per-package ownership model yet. Repository-level
permissions are the way to control who can publish.

## Installs pull from npmjs.org instead

The registry is configured globally but the package is scoped, or the other way round. `npm config
get registry` shows the global one; a scoped registry only applies to that scope:

```ini
@nitro:registry=https://repo.example.com/repositories/local/npm/
```

A package published as `example` rather than `@nitro/example` will not be found by that line.

## Still stuck

`npm --loglevel=verbose` prints every URL npm requests and every header it sends. Comparing that
against the server log usually settles it in one run.
