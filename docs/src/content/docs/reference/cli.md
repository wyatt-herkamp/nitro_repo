---
title: Command line
description: Every subcommand of the nitro_repo binary.
sidebar:
  order: 2
---

```text
nitro_repo <SUBCOMMAND> [OPTIONS]
```

`--version` and `--help` work everywhere, and `--help` on a subcommand prints its own options.

## `start`

Runs the server.

```sh
nitro_repo start --config nitro_repo.toml
```

| Option           | Default | What                            |
| ---------------- | ------- | ------------------------------- |
| `-c`, `--config` | none    | Path to the configuration file. |

`--config` is optional. Without it — or pointing at a file that does not exist — the server is
configured entirely from the environment, which is how the Docker image runs. A path that does not
exist is not an error.

Migrations run at startup, so a new version applies its own schema changes with no separate step.

## `save-config`

Writes a configuration file with every key at its default.

```sh
nitro_repo save-config --config nitro_repo.toml
```

| Option                 | Default           | What                                                |
| ---------------------- | ----------------- | --------------------------------------------------- |
| `-c`, `--config`       | `nitro_repo.toml` | Where to write.                                     |
| `-a`, `--add-defaults` | `false`           | Fill in missing keys of a file that already exists. |

Without `--add-defaults`, writing over an existing file is refused rather than silently replacing a
configuration someone spent time on.

With it, the existing file is read, merged with the defaults, and written back — which is how you
pick up keys added by a new version without diffing against the source.

The output doubles as the reference for what can be set. See
[Configuration file](/reference/configuration/).

## `config`

Opens an interactive editor for one section.

```sh
nitro_repo config --config nitro_repo.toml database
```

| Argument    | What                       |
| ----------- | -------------------------- |
| `<SECTION>` | Currently only `database`. |

It prompts for host, port, user, password and database name, tests the connection, and writes the
result back. Useful for the one section where a typo is not obvious until startup fails.

## `seed`

Fills a running instance with fixture artifacts over the real Maven and npm protocols.

```sh
nitro_repo seed --config seed.toml --write-example   # write the example
nitro_repo seed --config seed.toml                   # run it
```

| Option            | Default     | What                                               |
| ----------------- | ----------- | -------------------------------------------------- |
| `-c`, `--config`  | `seed.toml` | The seed configuration.                            |
| `--write-example` | `false`     | Write an example to `--config` instead of running. |

Re-running is safe; anything already present is left alone. See
[Seeding an instance](/admin/seeding/).

## `export`

Dumps internal metadata for tooling.

```sh
nitro_repo export open-api openapi.json
nitro_repo export repository-types ./types/
nitro_repo export repository-config-types ./configs/
```

| Value                     | What it writes                                                     |
| ------------------------- | ------------------------------------------------------------------ |
| `open-api`                | The OpenAPI document, as a single file.                            |
| `repository-types`        | One JSON file per repository type, into a directory.               |
| `repository-config-types` | A JSON Schema and a description per config type, into a directory. |

`open-api` is how the frontend's TypeScript types are generated — it needs no running server, so it
works in a build step. The other two write a directory rather than a file, which is worth knowing
before you pass a path ending in `.json` and get a directory of that name.

## `validate-frontend`

Checks the embedded frontend bundle.

```sh
nitro_repo validate-frontend
```

Only exists when the binary was built with `--features frontend`. It verifies there is an
`index.html` and a parseable `routes.json`, and exits non-zero if not.

Worth running after a build. Without the feature the server starts happily and answers every
non-API path with a `404`, which looks like a routing problem rather than a missing bundle.

## Exit codes

`0` on success, non-zero otherwise. `validate-frontend` exits `1` on a failed check.
