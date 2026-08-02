---
title: Developing
description: Setting up a working environment, running the tests, and what CI checks.
sidebar:
  order: 3
---

The short version of [`CONTRIBUTING.md`](https://github.com/wyatt-herkamp/nitro_repo/blob/main/CONTRIBUTING.md),
which is the file to trust if the two ever disagree.

## What you need

- **Rust** — the channel is pinned in `rust-toolchain.toml`, so `rustup` picks it up. Formatting
  additionally needs a nightly: `rustfmt.toml` sets `imports_granularity` and `group_imports`, which
  stable rustfmt ignores _without saying so_, so formatting on stable and on nightly disagree and
  fight each other.
- **Node** — the version is in `site/.node-version`.
- **Docker** — for Postgres and MinIO.
- **[`just`](https://github.com/casey/just)** — optional; every command below has a recipe.

## Backing services

```sh
just services-up
```

Starts MinIO and creates the test bucket. Postgres sits behind a profile, because `nr_tests.env`
points at `localhost:5432` and plenty of people already run one there — a second would just fail to
bind the port. If you need one:

```sh
docker compose -f docker-compose.dev.yml --profile postgres up -d --wait postgres
```

`nr_tests.env` holds the `DATABASE_URL` the tests use. It is gitignored; point it wherever you like.

## Running

```sh
cargo run -p nitro_repo -- save-config --config nitro_repo.toml
$EDITOR nitro_repo.toml     # at minimum, [database]
cargo run -p nitro_repo -- start --config nitro_repo.toml
```

Then open the instance and create the first administrator. To fill it with something to look at,
see [Seeding an instance](/admin/seeding/).

### Frontend

```sh
cd site
npm install
npm run dev
```

The dev server expects a backend on `http://localhost:6742`. If yours is elsewhere, put
`VITE_API_URL=http://host:port` in `site/.env.local`.

Two files under `site/` are **generated** — do not edit them by hand:

- `src/router/routes.json`, from the router. The backend reads it to decide which paths get the
  SPA's `index.html`, so a route missing from it is a hard 404 on refresh.
  `npm run generate-routes`.
- `src/types/api.d.ts`, from the OpenAPI document.
  `npm run export-openapi && npm run generate-api-types`.

## Tests

```sh
just test             # everything a laptop can run
just test-all         # the same, but a skipped backend or a missing database is a failure
just test-integration # only the end-to-end suite
```

Tests that need a service they cannot reach **skip with a message** rather than failing, so you are
never blocked by a backend you are not working on. CI sets `STORAGE_TESTS_REQUIRE_ALL=1` and
`NITRO_TESTS_REQUIRE_DB=1`, which turns every skip into a failure — otherwise a suite could quietly
stop running and nobody would notice.

The integration tests each create their own database and storage directory and drive the real
router in-process. They do not bind a port, so they run in parallel and there is no server to
start.

## Docs

These pages are Astro + Starlight, in `docs/`:

```sh
cd docs
npm install
npm run dev
```

Content is Markdown under `docs/src/content/docs/`. The sidebar is in `docs/astro.config.mjs`; a new
page is not reachable until it is listed there, or in the section it belongs to.

`npm run build` is what CI runs, and it fails on a broken internal link — which is the point.

## Before opening a pull request

```sh
just fmt      # cargo +nightly fmt --all, and prettier over site/
just lint     # clippy with -D warnings
just test-all
cd site && npm run type-check && npm run lint && npm run build
cd docs && npm run build
```

CI runs all of these. `deprecated = "deny"` is set workspace-wide, so a deprecation is a build
failure rather than a warning you can leave for later.

## Conventions worth knowing

- **`ahash::HashMap`, not `std::collections::HashMap`.** Enforced by a clippy `disallowed_types`
  rule, along with `parking_lot`'s `Mutex` and `RwLock`.
- **Comments explain why, not what.** The codebase leans heavily on doc comments that record the
  bug a piece of code exists to prevent. Keep that up — it is what makes the tests legible.
- **A test that skips must say so.** Silent skipping is how a suite stops running without anyone
  noticing.

## Where to start

Issues labelled [help wanted](https://github.com/wyatt-herkamp/nitro_repo/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22).
The [comparison page](/about/comparison/) lists the gaps that are open by design — repository
grouping, npm proxying and cleanup policies are all genuinely wanted.
