---
title: Authenticating with Docker
description: docker login, the bearer token exchange, and CI.
sidebar:
  order: 2
---

## `docker login`

```sh
docker login repo.example.com
# or, with a hostname on the repository:
docker login docker.example.com
```

Use your username, and an **auth token** from **Profile → Tokens** as the password. Your account
password works too, but a token is what you want: it can be scoped to one repository and revoked on
its own.

## What actually happens

Docker never sends your credentials to the registry endpoints. It performs the standard bearer
exchange:

1. `GET /v2/` — the capability probe.
2. If the registry wants credentials, it answers `401` with a challenge naming where to get a token:

   ```text
   WWW-Authenticate: Bearer realm="https://repo.example.com/api/docker/token",
                            scope="repository:local/docker/myimage:pull,push"
   ```

3. Docker calls that realm with Basic credentials and gets a short-lived token back.
4. Every subsequent request carries `Authorization: Bearer <token>`.

The token is an ordinary Nitro Repo auth token — the same rows `Profile → Tokens` lists — scoped to
the repository in the challenge and **expiring after 30 minutes**. Docker fetches a new one whenever
it gets a `401`, so the short life is invisible. Revoking a token from the profile UI logs the
Docker client out too.

The realm is built from the host the client is already talking to, not from `site.app_url`. A
registry on a custom domain may be the only address that resolves from wherever Docker is running.

## Anonymous pulls

A `Public` repository serves pulls with no credentials at all — no challenge, no token. Only writes
require authentication.

A `Private` or `Hidden` one challenges every read, which is what makes `docker pull` fetch a token
rather than give up.

## CI

```sh
echo "$NITRO_TOKEN" | docker login repo.example.com -u "$NITRO_USER" --password-stdin
docker push repo.example.com/local/docker/myimage:$CI_COMMIT_SHA
```

`--password-stdin` rather than `-p`: a token on the command line ends up in the process list and in
shell history.

Give the CI token `Read` and `Write` on that repository and nothing else.

## Troubleshooting

**`http: server gave HTTP response to HTTPS client`** — the daemon will not talk to a plain-HTTP
registry. Put the instance behind TLS, or add the host to `insecure-registries` in
`/etc/docker/daemon.json` and restart the daemon. `localhost` is exempt.

**The push says `Layer already exists` and then stops, with no digest line and no error** — the
daemon could not reach the realm in the challenge. Check that `site.app_url` (or the repository's
hostname) is an address the client can resolve, _including the port_.

**`denied: you do not have push access to this registry`** — the token authenticated but is not
scoped to this repository with `Write`, or the user lacks that action. Both are checked.

**`unauthorized` on a pull of a public image** — the repository is `Private` or `Hidden` rather than
`Public`. Visibility is on the repository, not in its config.

## Next

- [Pushing and pulling](/repositories/docker/pushing/)
- [API tokens and scopes](/admin/tokens/)
