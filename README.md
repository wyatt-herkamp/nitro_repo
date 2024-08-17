# nitro_repo [![Documentation](https://img.shields.io/static/v1?label=nitro-repo.kingtux.dev&message=Here&style=for-the-badge&color=green)](https://nitro-repo.kingtux.dev/) [![Powered By Actix](https://img.shields.io/badge/Powered%20By-Actix-red?style=for-the-badge&logo=rust)](https://github.com/actix/actix-web)

[![issues](https://img.shields.io/github/issues/wherkamp/nitro_repo/help%20wanted)](https://github.com/wherkamp/nitro_repo/issues)

Nitro Repo is an open source free artifact manager. Written with a Rust back end and a Vue front end to create a fast
and modern experience.

### History

After years of using Nexus and then a bit of time of using StrongBox I decided I should design my own Artifact Manager
to create a fast and modern experience.

### Technical Design

- Backend or the heart of nitro_repo
  - SQLX for Postgres
  - Axum for HTTP Server
- Frontend
  - Vue
  - Vite

### Crates
- crates/core
  - Lays out some shared data types between different modules.
- crates/macros
  - Macros used by the other crates. To prevent writing so much code
- crates/storages
  - This layer provides different ways storing the artifacts that nitro-repo hosts

