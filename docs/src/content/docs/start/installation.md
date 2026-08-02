---
title: Installation
description: Run Nitro Repo with Docker Compose, from a release binary, or from source.
sidebar:
  order: 2
---

Nitro Repo needs **Postgres**. That is the only hard external dependency — sessions live in an
embedded database, and artifact storage is whatever you configure later.

## Docker Compose

The quickest working instance. `docker-compose.yml` in the repository brings up the server and the
Postgres it needs:

```sh
curl -O https://raw.githubusercontent.com/wyatt-herkamp/nitro_repo/main/docker-compose.yml
docker compose up -d
```

Then open `http://localhost:6742` and continue with [First run](/start/first-run/).

Two things to change before this is anything but a trial:

- The Postgres password appears twice in the file — once for the database and once for the server.
  Change both.
- `image:` is `:latest` against the beta builds. Pin a tag before you depend on it.

The compose file configures the server entirely through environment variables, so there is no file
to create before the first `up`. If you would rather keep settings in TOML, mount one at
`/etc/nitro-repo/nitro-repo.toml` — that is the path the image's entrypoint passes to `--config`,
and a missing file there is not an error.

:::caution[The prefix contains a hyphen]
Environment variables are read with the prefix `NITRO-REPO_` — with a hyphen, not an underscore. A
variable spelled `NITRO_REPO_DATABASE_HOST`, or one with no prefix at all, is ignored silently
rather than rejected. See [Configuration file](/reference/configuration/#environment-variables).
:::

### Your own Postgres

Drop the `postgres` service and its `depends_on`, and point the server at yours:

```yaml
services:
  nitro_repo:
    image: "git.kingtux.dev/wherkamp/nitro_repo/nitro_repo:latest"
    restart: unless-stopped
    environment:
      NITRO-REPO_DATABASE_HOST: db.internal
      NITRO-REPO_DATABASE_PORT: "5432"
      NITRO-REPO_DATABASE_USER: nitro_repo
      NITRO-REPO_DATABASE_PASSWORD: "…"
      NITRO-REPO_DATABASE_DATABASE: nitro_repo
      NITRO-REPO_SESSIONS_DATABASE_LOCATION: /data/sessions.redb
      NITRO-REPO_STAGING_STAGING_DIR: /data/staging
      NITRO-REPO_SUGGESTED_LOCAL_STORAGE_PATH: /data/storages/
    ports:
      - "6742:6742"
    volumes:
      - nitro_repo_data:/data/
```

The database is created for you if it is empty — migrations run at startup — but the database
itself and its role must already exist.

Set `SESSIONS_DATABASE_LOCATION` and `STAGING_STAGING_DIR` under the volume, as above. Their
defaults are relative to the working directory, which in the image is not persisted, so leaving
them alone means every user is signed out whenever the container is replaced.

## Release binary

Grab a build from [Releases](https://github.com/wyatt-herkamp/nitro_repo/releases) and unpack it
somewhere sensible — `/opt/nitro_repo` is a reasonable choice.

```sh
cd /opt/nitro_repo
./nitro_repo save-config --config nitro_repo.toml
$EDITOR nitro_repo.toml   # at minimum, the [database] section
./nitro_repo start --config nitro_repo.toml
```

`save-config` writes every key with its default value, which doubles as the reference for what can
be set. [Configuration file](/reference/configuration/) explains the sections.

### Running it as a service

Nitro Repo does not ship a unit file. This one works:

```ini
# /etc/systemd/system/nitro_repo.service
[Unit]
Description=Nitro Repo
After=network.target postgresql.service

[Service]
Type=simple
User=nitro_repo
WorkingDirectory=/opt/nitro_repo
ExecStart=/opt/nitro_repo/nitro_repo start --config /opt/nitro_repo/nitro_repo.toml
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```sh
systemctl daemon-reload
systemctl enable --now nitro_repo
```

The working directory matters: `sessions.database_location` and `staging.staging_dir` default to
paths relative to it.

## From source

You need the Rust toolchain — the channel is pinned in `rust-toolchain.toml`, so `rustup` picks the
right one — and Node, whose version is in `site/.node-version`.

The frontend is compiled into the binary rather than served from disk, so it has to be built first
and the backend has to be built with the `frontend` feature:

```sh
git clone https://github.com/wyatt-herkamp/nitro_repo.git
cd nitro_repo

cd site && npm ci && npm run build && cd ..
FRONTEND_DIST=$PWD/site/dist cargo build --release --features frontend

./target/release/nitro_repo validate-frontend
```

`validate-frontend` checks that the embedded bundle has an `index.html` and a usable `routes.json`.
It is worth running: without the `frontend` feature the server builds and starts perfectly happily
and then answers every non-API path with a 404, which looks like a routing problem rather than a
missing build.

Building the backend alone — for API-only use, or while working on the server — is just
`cargo build --release`.

See [Developing](/develop/) for the full working environment, including the test services.

## Behind a reverse proxy

Nitro Repo can terminate TLS itself:

```toml
[web_server.tls]
private_key = "/etc/letsencrypt/live/example.com/privkey.pem"
certificate_chain = "/etc/letsencrypt/live/example.com/fullchain.pem"
```

Most people front it with nginx or Caddy instead. Two things to get right either way:

- **Set `site.app_url`** to the URL your users actually type. Anything that has to hand a client an
  absolute URL reads it, and npm's browser login is the case that will bite you: with no `app_url`
  it answers `npm login` with a 503 telling the user to pass `--auth-type=legacy`, because a login
  link that is only valid inside your network is worse than no link. The comment in the config file
  says the URL is otherwise taken from the request; nothing currently does that, so treat this as
  required rather than optional.
- **Raise the proxy's body limit.** Nitro Repo's own limit is `web_server.max_upload`, 100 MiB by
  default; nginx's `client_max_body_size` defaults to 1 MiB and will reject a large jar before
  Nitro Repo ever sees it.

Also set `site.is_https = true` when TLS is terminated upstream, so session cookies are marked
secure.

## Next

[First run](/start/first-run/) — creating the administrator, a storage and a repository.
