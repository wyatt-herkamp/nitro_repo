---
title: Cargo
description: A hosted Cargo registry speaking the sparse index protocol — what is supported, and what is not there yet.
sidebar:
  order: 1
---

A Cargo repository is a hosted registry. Point `.cargo/config.toml` at its index and
`cargo publish`, `cargo add` and `cargo build` work as they do against crates.io.

```toml
# .cargo/config.toml
[registries.nitro]
index = "sparse+https://repo.example.com/repositories/local/crates/index/"
```

:::caution[The trailing slash is not optional]
Cargo appends the index path to this URL. Without the slash the last segment is replaced rather
than appended, and every lookup 404s on a crate that exists.
:::

## What works

|                    |                                                                                 |
| ------------------ | ------------------------------------------------------------------------------- |
| **Sparse index**   | The HTTP index, default since Rust 1.68. No git server involved.                |
| **Publish**        | `cargo publish`, including the `publish = ["nitro"]` manifest gate.             |
| **Download**       | `cargo add`, `cargo build`, `cargo fetch`, with `cksum` verified by the client. |
| **Yank**           | `cargo yank` and `cargo yank --undo`.                                           |
| **Owners**         | `cargo owner --list`, `--add` and `--remove`.                                   |
| **Search**         | `cargo search`.                                                                 |
| **Renamed deps**   | `package = "…"` in the index, so `foo = { package = "bar" }` resolves.          |
| **Feature syntax** | Index schema 2, so `dep:` and `foo?/bar` features are expressible.              |
| **Access control** | Reads honour repository visibility; writes require a `Write` action.            |
| **Custom domains** | `config.json` reports whichever host the client used.                           |

## What is not there

- **No proxying.** There is no `Proxy` mode, so a registry cannot mirror crates.io. Keep crates.io
  configured alongside yours.
- **No git index.** Only the sparse protocol. A toolchain older than Rust 1.68 cannot use this
  registry.
- **No `cargo login` handshake.** There is no credential-provider flow; paste an API token, as
  below.
- **No delete.** A published version can be yanked, never removed — which is what crates.io does,
  and what every lockfile pinning it depends on.

## Crate names

The registry enforces Cargo's own rules: at most 64 characters, starting with a letter, and made up
of `a-z`, `A-Z`, `0-9`, `-` and `_`. A name outside that is refused at publish time rather than
stored as something nobody could depend on.

## The index

A crate's versions are served as newline-delimited JSON — one object per version, in publication
order:

```text
GET /repositories/local/crates/index/ni/tr/nitro-example
```

```json
{"name":"nitro-example","vers":"0.1.0","deps":[],"cksum":"…","features":{},"yanked":false,"v":2}
{"name":"nitro-example","vers":"0.2.0","deps":[],"cksum":"…","features":{},"yanked":true,"v":2}
```

The path prefix comes from the crate name's length, which is how Cargo keeps one directory from
holding every crate in the registry:

| Name length | Path                   | Example               |
| ----------- | ---------------------- | --------------------- |
| 1           | `1/{name}`             | `1/a`                 |
| 2           | `2/{name}`             | `2/ab`                |
| 3           | `3/{first}/{name}`     | `3/f/foo`             |
| 4 or more   | `{c1c2}/{c3c4}/{name}` | `ni/tr/nitro-example` |

The name is lowercased for the path even though the crate name itself is not. A request whose path
does not match the name it ends with is refused rather than served — otherwise one crate could be
fetched under another's path, and Cargo caches by path.

The index is generated from what the registry holds rather than stored, and every response carries
an `ETag`. Cargo revalidates constantly, so a `304` is the common case.

## Next

- [Authenticating](/repositories/cargo/authenticating/) — tokens, and CI.
- [Publishing](/repositories/cargo/publishing/) — publish, yank, owners.
- [Configuration](/repositories/cargo/configuration/)
