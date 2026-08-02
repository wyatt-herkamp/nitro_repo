---
title: Authenticating with npm
description: Browser login, legacy login, and putting a token in .npmrc for CI.
sidebar:
  order: 2
---

## Pointing npm at the registry

```ini
# .npmrc
registry=https://repo.example.com/repositories/local/npm/
```

Or scope it, which is what you want if you also use the public registry:

```ini
@nitro:registry=https://repo.example.com/repositories/local/npm/
```

The trailing slash matters. See [the note on the overview page](/repositories/npm/#what-works).

## `npm login`

```sh
npm login --registry=https://repo.example.com/repositories/local/npm/
```

npm 9 and later default to `--auth-type=web`: the CLI asks the registry to start a login session,
prints a URL, and waits. You open the URL, sign in to Nitro Repo normally, approve, and the CLI
picks up a token.

```text
npm notice Log in on https://repo.example.com/npm/login/2f0c…
```

Under the hood: npm `POST`s `/-/v1/login` and gets back a `loginUrl` and a `doneUrl`. Approving in
the browser mints a repository-scoped token; npm polls `doneUrl`, gets `202` with a `retry-after`
while the session is pending, and `200` with the token once it is approved. The token is handed out
exactly once.

Sessions last **15 minutes** and live in memory. A server restart drops any in flight, which costs
one re-run of `npm login`.

:::caution[`app_url` must be set]
Both URLs go to a browser and to npm, so a relative one is useless. With no `site.app_url`
configured, the registry answers `npm login` with a `503` telling you to use `--auth-type=legacy`
rather than handing out a link to nowhere. See [Installation](/start/installation/#behind-a-reverse-proxy).
:::

## `npm login --auth-type=legacy`

The older CouchDB-style flow: the CLI prompts for a username and password and `PUT`s them to
`/-/user/org.couchdb.user:{username}`. The registry validates them and returns a token, which npm
writes to `.npmrc`.

Use it when the browser flow is not an option — a headless machine, or an instance with no
`app_url`.

## CI

Do not run `npm login` in CI. Mint a token in **Profile → Tokens** and write it straight into
`.npmrc`:

```ini
//repo.example.com/repositories/local/npm/:_authToken=${NITRO_TOKEN}
@nitro:registry=https://repo.example.com/repositories/local/npm/
```

npm expands `${NITRO_TOKEN}` from the environment, so the file itself holds no secret and can be
committed.

The key is the registry URL with the protocol stripped, and it has to match the registry URL
exactly — including the trailing slash. A mismatch here is silent: npm simply does not send the
token, and you get a 401 that looks like the token is wrong.

Give the token `WriteRepository`, or a per-repository `Write` scope on just this registry. See
[API tokens and scopes](/admin/tokens/).

## Checking

```sh
npm whoami --registry=https://repo.example.com/repositories/local/npm/
```

Answers with your username when the credentials work, and `401` when they do not.

`npm ping` checks the registry is reachable at all, without authentication:

```sh
npm ping --registry=https://repo.example.com/repositories/local/npm/
```

## Reads on a private registry

A `Public` registry needs no credentials to install from. A `Private` one needs a token with read
access, configured exactly as above — npm sends `_authToken` on reads as well as writes.
