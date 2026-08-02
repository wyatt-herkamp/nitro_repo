# nitro_repo [![Documentation](https://img.shields.io/static/v1?label=nitro-repo.kingtux.dev&message=Here&style=for-the-badge&color=green)](https://nitro-repo.kingtux.dev/) [![Powered By Axum](https://img.shields.io/badge/Powered%20By-Axum-black?style=for-the-badge&logo=rust)](https://github.com/tokio-rs/axum)

[![issues](https://img.shields.io/github/issues/wherkamp/nitro_repo/help%20wanted)](https://github.com/wherkamp/nitro_repo/issues)

Nitro Repo is a free and open source artifact manager, with a Rust backend and a Vue frontend.

> The badge said "Powered By Actix" until recently. 2.0 moved to Axum and Sea-ORM to SQLx; the badge
> had simply never caught up.

### History

After years of using Nexus, and a while on Strongbox, I decided to design my own artifact manager for
a faster and more modern experience.

### What it does

- **Maven** — hosted and proxy repositories, generated `maven-metadata.xml` (including timestamped
  snapshots), checksum verification on upload and generation on demand, and enforced push rules.
- **npm** — a hosted registry: scoped packages, dist-tags, publish, unpublish, deprecate, search,
  and both login flows including the browser one npm 9+ defaults to.
- **Cargo** — a hosted registry over the sparse index: publish, yank, owners, search, and renamed
  dependencies.
- **Docker** — a hosted OCI registry: push and pull, multi-arch images, referrers, and the bearer
  token exchange `docker login` uses. Addressed on a domain of its own or by an image-name prefix.
- **Storage** — the local filesystem, a self-contained object format (FileSystemV2), or S3 and
  anything S3-compatible.
- **Search** — a small query language (`crates/aql`) over projects and versions, exposed both as
  plain text search and as a full query syntax.

### Technical design

- Backend
  - Axum for the HTTP server
  - SQLx against Postgres
  - utoipa for the OpenAPI document, from which the frontend's types are generated
- Frontend
  - Vue 3
  - Vite

### Crates

| Path              | What it is                                                                    |
| ----------------- | ----------------------------------------------------------------------------- |
| `nitro_repo/`     | The server: HTTP, repository types, authentication. `main.rs` is only the CLI  |
| `crates/core/`    | Shared types, the database entities, and the migrations                       |
| `crates/storage/` | Storage backends — Local, FileSystemV2, S3                                    |
| `crates/aql/`     | The query language behind search. Depends on nothing but serde and thiserror  |
| `crates/macros/`  | Macros the other crates use                                                   |
| `crates/nr-api/`  | A client for the API                                                          |

### Running it

The published image, with the Postgres it needs and a volume for its data:

```sh
docker compose up -d
```

Then open `http://localhost:6742` and create the first administrator. Change the password in
`docker-compose.yml` before this is anything but a trial.

Building it yourself, and everything else you would need to work on it, is in
[CONTRIBUTING.md](CONTRIBUTING.md). To fill an instance with something to look at,
`nitro_repo seed --config seed.toml` deploys a suite of artifacts over the real protocols.

### Documentation

[nitro-repo.kingtux.dev](https://nitro-repo.kingtux.dev/) — installation, configuration, the
per-format guides, and the API reference. The source is in [`docs/`](docs/) (Astro + Starlight).

### Status

2.0 is in beta. Breaking changes are still on the table.

### Contributors

[![Contributors](https://contrib.rocks/image?repo=wyatt-herkamp/nitro_repo)](https://github.com/wyatt-herkamp/nitro_repo/graphs/contributors)

Thanks to everyone who has sent a patch. Contributions of any size are welcome —
[CONTRIBUTING.md](CONTRIBUTING.md) has what you need to get a working environment.
