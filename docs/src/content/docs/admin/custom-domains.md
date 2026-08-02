---
title: Custom domains
description: Serve a repository from its own hostname, with no /repositories prefix in the URL.
sidebar:
  order: 2
---

By default a repository lives under a path:

```
https://nitro.example.com/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.jar
```

Attach a domain to it and the same artifact is served from the root of that host:

```
https://maven.example.com/dev/kingtux/tms/1.0.0/tms-1.0.0.jar
```

Nothing else changes. The repository keeps its `/repositories/{storage}/{repository}` URL — a custom
domain is an additional way in, not a replacement — and renaming the repository does not break the
domain, because the two are unrelated.

## Adding one

**Admin → Repositories → _your repository_ → Main → Custom domains.** Type the bare hostname and add
it. It takes effect immediately; no restart.

A hostname is unique across the whole instance, so two repositories cannot claim the same one, and
adding or removing one needs the same permission as creating a repository.

### Before it will work

Adding the domain here only tells Nitro Repo what to do with a request once it arrives. Two things
have to be true first.

1. **DNS points at this instance.** A `CNAME` or `A` record for `maven.example.com` resolving to
   wherever Nitro Repo is reachable.
2. **TLS covers the name.** Either a certificate that includes it, or a reverse proxy in front that
   terminates TLS for it.

### Behind a reverse proxy

The routing decision is made from the request's `Host`, so the proxy has to pass the original one
through. This is the single most common reason a domain "does nothing":

```nginx
location / {
    proxy_pass http://127.0.0.1:6742;
    proxy_set_header Host $host;
}
```

Traefik and Caddy preserve the `Host` by default.

If your proxy rewrites `Host` and sends the original in `X-Forwarded-Host` instead, set
`trust_forwarded_host` in [`[security]`](/reference/configuration/#security). It is off by default
because any client can send that header — trusting it without a proxy in front that overwrites it
would let a caller choose which repository their request lands in.

## What a domain does not take over

`/api`, `/badge` and `/repositories` are matched before the host is considered, on every host. So on
`maven.example.com`:

- `/api/…` is still the API, and the web UI still loads;
- `/repositories/local/releases/…` still serves the same artifacts;
- everything else is an artifact path inside the repository the domain belongs to.

The trade-off is that an artifact whose path begins with one of those segments — a Maven group id of
`api.example`, say — is not reachable over the custom domain. Use the repository's normal URL for
those.

## Using it

### Maven

```xml
<repository>
  <id>example</id>
  <url>https://maven.example.com</url>
</repository>
```

### npm

```
npm config set registry https://npm.example.com
```

npm rewrites `dist.tarball` to whatever registry it publishes to, so a package published over a
custom domain records tarball URLs on that domain. Both forms are accepted, and a package published
one way is installable the other.

## Removing one

Remove it from the same card. Requests to that host stop reaching the repository straight away, and
the name becomes available again — as it also does when the repository itself is deleted. Anything
still pulling from that URL will break, so change those first.
