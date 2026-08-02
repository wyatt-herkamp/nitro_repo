# Contributing

## What you need

- **Rust** — the toolchain is pinned in `rust-toolchain.toml`, so `rustup` picks it up. Formatting
  additionally needs a nightly (`rustup toolchain install nightly`): `rustfmt.toml` sets
  `imports_granularity` and `group_imports`, which stable rustfmt ignores *without saying so*, so
  formatting on stable and on nightly disagree and fight each other.
- **Node** — the version is in `site/.node-version`.
- **Docker** — for Postgres and MinIO. You can point at your own instead; see below.
- **[`just`](https://github.com/casey/just)** — optional, but every command below has a recipe.

There is no MySQL and no C++ connector. That requirement, along with the Node 16 and the `.env` file
this document used to describe, predates the 2.0 rewrite by several years.

## Backing services

```sh
just services-up
```

That starts MinIO and creates the test bucket. Postgres sits behind a profile, because
`nr_tests.env` points at `localhost:5432` and plenty of people already run one there — a second
would just fail to bind the port. If you need one:

```sh
docker compose -f docker-compose.dev.yml --profile postgres up -d --wait postgres
```

`nr_tests.env` holds the `DATABASE_URL` the tests use. Point it wherever you like.

## Running the tests

```sh
just test             # everything a laptop can run
just test-all         # the same, but a skipped backend or a missing database is a failure
just test-integration # only the end-to-end suite
```

Tests that need a service they cannot reach **skip with a message** rather than failing, so you are
never blocked by a backend you are not working on. CI sets `STORAGE_TESTS_REQUIRE_ALL=1` and
`NITRO_TESTS_REQUIRE_DB=1`, which turns every skip into a failure — otherwise a suite could quietly
stop running and nobody would notice.

The integration tests each create their own database and storage directory, and drive the real
router in-process. They do not bind a port, so they run in parallel and there is no server to start.

## Running it

```sh
cargo run -p nitro_repo -- save-config --config nitro_repo.toml
$EDITOR nitro_repo.toml     # at minimum, the [database] section
cargo run -p nitro_repo -- start --config nitro_repo.toml
```

Then open the instance and create the first administrator.

### Filling it with something to look at

An empty instance is hard to judge — browsing, search, badges and metadata merging all behave
differently with one artifact than with forty. This deploys a suite of Maven artifacts and npm
packages over the real protocols:

```sh
cargo run -p nitro_repo -- seed --config seed.toml --write-example
$EDITOR seed.toml           # the password, and the storage path
just seed seed.toml
```

Re-running is safe; anything already present is left alone.

## Frontend

```sh
cd site
npm install
npm run dev
```

The dev server expects a backend on `http://localhost:6742`. If yours is elsewhere, put
`VITE_API_URL=http://host:port` in `site/.env.local`.

Two things under `site/` are **generated** — do not edit them by hand:

- `src/router/routes.json`, from the router. The backend reads it to decide which paths get the
  SPA's index.html, so a route missing from it is a hard 404 on refresh. `npm run generate-routes`.
- `src/types/api.d.ts`, from the OpenAPI document.
  `npm run export-openapi && npm run generate-api-types`.

## Before you open a pull request

```sh
just fmt      # cargo +nightly fmt --all, and prettier over site/
just lint     # clippy with -D warnings
just test-all
cd site && npm run type-check && npm run lint && npm run build
```

CI runs all of these. `deprecated = "deny"` is set workspace-wide, so a deprecation is a build
failure rather than a warning you can leave for later.

## Layout

| Path              | What it is                                                                    |
| ----------------- | ----------------------------------------------------------------------------- |
| `nitro_repo/`     | The server: HTTP, repository types, authentication. `main.rs` is only the CLI  |
| `crates/core/`    | Shared types, the database entities, and the migrations                       |
| `crates/storage/` | Storage backends — Local, FileSystemV2, S3                                    |
| `crates/aql/`     | The query language behind search. Depends on nothing but serde and thiserror  |
| `crates/macros/`  | Macros the other crates use                                                   |
| `crates/nr-api/`  | A client for the API                                                          |
| `site/`           | The frontend                                                                  |
