---
title: Configuration file
description: Every section of nitro_repo.toml, and the environment variables that stand in for them.
sidebar:
  order: 1
---

`nitro_repo save-config --config nitro_repo.toml` writes a file with every key at its default. That
file is the authoritative reference; this page explains what the keys mean.

Repository and storage settings are **not** here — they live in Postgres and are edited through the
UI, because they change while the server is running.

## Where settings come from

Two sources: the file passed to `--config`, and the environment.

**The file wins.** If a section is present in the file, the environment is ignored for the whole of
that section — the merge happens per top-level section, not per key. So a file containing

```toml
[database]
host = "localhost"
```

means `NITRO-REPO_DATABASE_PASSWORD` has no effect, because `[database]` came from the file.

The practical consequence: use one or the other for a given section. Mixing them produces
surprises. Docker deployments should have no file at all, which is why a missing `--config` path is
not an error.

### Environment variables

The prefix is `NITRO-REPO_`, with a **hyphen**. Nested keys are joined with underscores and the
whole name is upper case:

```text
NITRO-REPO_DATABASE_HOST=postgres
NITRO-REPO_WEB_SERVER_BIND_ADDRESS=0.0.0.0:6742
NITRO-REPO_SESSIONS_DATABASE_LOCATION=/data/sessions.redb
NITRO-REPO_SUGGESTED_LOCAL_STORAGE_PATH=/data/storages/
```

:::caution
A variable with the wrong prefix — `NITRO_REPO_…`, or none at all — is **ignored silently**. It is
not rejected and nothing is logged, so a typo looks exactly like the setting having no effect.

The hyphen also means most shells cannot `export` these directly. Docker Compose, systemd's
`Environment=`, and Kubernetes all set them without trouble.
:::

## `mode`

`Debug` or `Release`. Defaults to whichever the binary was built as. Affects logging verbosity and
error detail.

## `[web_server]`

| Key               | Default        | What                                     |
| ----------------- | -------------- | ---------------------------------------- |
| `bind_address`    | `0.0.0.0:6742` | Address and port to listen on.           |
| `open_api_routes` | `true`         | Serve `/open-api-doc-raw` and `/scalar`. |
| `max_upload`      | `100MiB`       | Largest request body accepted.           |

`max_upload` takes a size — `100`, `100b`, `100KiB`, `100MiB` — or the string `unlimited`. A bare
number is bytes.

Raise it if you publish large artifacts, and remember to raise the limit on any proxy in front:
nginx's `client_max_body_size` defaults to 1 MiB and rejects the request before Nitro Repo sees it.

### `[web_server.tls]`

Optional. Terminate TLS in the server rather than upstream.

| Key                 | What                     |
| ------------------- | ------------------------ |
| `private_key`       | Path to the private key. |
| `certificate_chain` | Path to the full chain.  |

```toml
[web_server.tls]
private_key = "/etc/letsencrypt/live/example.com/privkey.pem"
certificate_chain = "/etc/letsencrypt/live/example.com/fullchain.pem"
```

## `[database]`

Postgres. Required — the server does not start without one.

| Key        | Default      | What                                     |
| ---------- | ------------ | ---------------------------------------- |
| `host`     | `localhost`  | Hostname. May be `host:port`.            |
| `port`     | `5432`       | Port. Taken from `host` if not set here. |
| `user`     | `postgres`   | Role to connect as.                      |
| `password` | `password`   | Its password.                            |
| `database` | `nitro_repo` | Database name. Also accepted as `name`.  |

The database and role must already exist; the schema does not, and migrations run at startup.

`nitro_repo config --config nitro_repo.toml database` prompts for these and tests the connection
before writing, which catches a typo now rather than at the next start.

## `[site]`

| Key           | Default                            | What                              |
| ------------- | ---------------------------------- | --------------------------------- |
| `app_url`     | none                               | The URL users type. **Set this.** |
| `name`        | `Nitro Repo`                       | Shown in the UI.                  |
| `description` | `An Open Source artifact manager.` | Shown in the UI.                  |
| `is_https`    | `false`                            | Marks session cookies secure.     |

Set `app_url` to the externally reachable URL. Anything that must hand a client an absolute URL
reads it, and npm's browser login refuses to run without one — it answers `npm login` with a `503`
rather than issuing a link that only resolves inside your network.

Set `is_https = true` whenever users reach the instance over HTTPS, including when TLS is
terminated by a proxy.

## `[sessions]`

Browser sessions, kept in an embedded database rather than in Postgres.

| Key                 | Default           | What                                        |
| ------------------- | ----------------- | ------------------------------------------- |
| `lifespan`          | `86400`           | Seconds a session lasts. One day.           |
| `cleanup_interval`  | `3600`            | Seconds between sweeps of expired sessions. |
| `database_location` | `./sessions.redb` | Path to the session database.               |

`database_location` is **relative to the working directory**. In a container that is usually not a
volume, so leaving it alone means every user is signed out whenever the container is replaced. Put
it somewhere persistent.

## `[security]`

| Key                          | Default | What                                        |
| ---------------------------- | ------- | ------------------------------------------- |
| `allow_basic_without_tokens` | `false` | Declared but not read. Has no effect today. |

`allow_basic_without_tokens` appears in the generated file and in this table because it is in the
config struct, but nothing consults it. To require tokens for deploys, use the per-repository
`must_use_auth_token_for_push` push rule instead — that one is enforced.

### `[security.password_rules]`

| Key                 | Default |
| ------------------- | ------- |
| `min_length`        | `8`     |
| `require_uppercase` | `true`  |
| `require_lowercase` | `true`  |
| `require_number`    | `true`  |
| `require_symbol`    | `true`  |

Applied when a password is set or changed. Removing the section entirely disables the checks.

## `[staging]`

| Key                 | Default     | What                                         |
| ------------------- | ----------- | -------------------------------------------- |
| `staging_dir`       | `./staging` | Scratch space for in-progress uploads.       |
| `time_till_cleanup` | `3600`      | Seconds before an abandoned upload is swept. |

`staging_dir` is relative to the working directory, with the same container caveat as the session
database.

## `[email]`

Optional. Only needed for password-reset emails; without it, that endpoint has nowhere to send
anything and an administrator sets passwords directly instead.

| Key          | What                         |
| ------------ | ---------------------------- |
| `username`   | SMTP username.               |
| `password`   | SMTP password.               |
| `host`       | SMTP host.                   |
| `encryption` | `NONE`, `StartTLS` or `TLS`. |
| `from`       | Envelope sender.             |
| `reply_to`   | Optional reply-to.           |

## `suggested_local_storage_path`

A path pre-filled into the "create a storage" form. Purely a convenience, so an operator does not
have to know where the data volume is mounted.

## `[log]`

Logging and OpenTelemetry. The generated file has the full structure; the shape is:

```toml
[log.levels]
default = "Info"

[log.levels.others]
nitro_repo = "Debug"
sqlx = "Warn"

[log.loggers.console]
type = "Console"

[log.loggers.console.config]
pretty = false
include_time = true
```

Three loggers are configured by default — `console`, `file` (a daily rolling file) and `app` (OTLP,
disabled) — and any of them can be removed. Levels are `Trace`, `Debug`, `Info`, `Warn`, `Error`,
with per-target overrides under `others`.

`[log.metrics]` configures OTLP metrics export, off by default.

## A minimal file

Everything else has a working default:

```toml
[database]
host = "localhost"
port = 5432
user = "nitro_repo"
password = "…"
database = "nitro_repo"

[site]
app_url = "https://repo.example.com"
is_https = true

[sessions]
database_location = "/var/lib/nitro_repo/sessions.redb"

[staging]
staging_dir = "/var/lib/nitro_repo/staging"
```
