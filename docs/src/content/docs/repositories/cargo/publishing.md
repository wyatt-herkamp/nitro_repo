---
title: Publishing to Cargo
description: cargo publish, yank, and owners.
sidebar:
  order: 3
---

## Publishing

Cargo refuses to publish anywhere unless the crate says so. Add the registry to the manifest:

```toml
# Cargo.toml
[package]
name = "nitro-example"
version = "0.1.0"
description = "…"
license = "MIT"
publish = ["nitro"]
```

Then:

```sh
cargo publish --registry nitro
```

Without `publish`, Cargo targets crates.io and stops with "the registry is not listed". With
`publish = false` it refuses everywhere.

### What happens

Cargo packages the crate, then `PUT`s it to `/api/v1/crates/new` as two length-prefixed frames — the
metadata JSON, then the `.crate` file. The registry:

1. checks the name and version are well formed;
2. refuses the publish if that version already exists;
3. computes the `cksum` from the bytes it received, not from anything the client claimed;
4. stores the `.crate` and writes the index record.

Cargo then polls the index until the new version shows up, which is the "waiting for … to be
available" line.

### Re-publishing is refused

```text
error: failed to publish to registry at …
Caused by: crate version `nitro-example@0.1.0` already exists.
```

A published version is immutable. Every lockfile that pins it, and every `cksum` in the index,
assumes the bytes never change. Publish a new version instead.

## Yanking

```sh
cargo yank --registry nitro --version 0.1.0 nitro-example
cargo yank --registry nitro --version 0.1.0 --undo nitro-example
```

Yanking sets `"yanked": true` in the index. New resolutions skip the version; a lockfile that
already pins it keeps working, and the `.crate` stays downloadable. That is the whole difference
between yank and delete — and delete is not offered.

## Owners

```sh
cargo owner --registry nitro --list nitro-example
cargo owner --registry nitro --add someone nitro-example
cargo owner --registry nitro --remove someone nitro-example
```

Whoever publishes a crate first becomes its owner. Adding or removing an owner needs more than
publish rights: the caller must already manage the crate, or be able to administer the repository.
Repository `Write` alone is not enough, because handing a crate to a third party is a stronger act
than pushing a version of it.

## Searching

```sh
cargo search --registry nitro nitro
```

Matches the crate name and description.

## Next

- [Configuration](/repositories/cargo/configuration/)
