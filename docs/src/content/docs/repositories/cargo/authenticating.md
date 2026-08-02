---
title: Authenticating with Cargo
description: Tokens, credentials.toml, and CI.
sidebar:
  order: 2
---

## Pointing Cargo at the registry

```toml
# .cargo/config.toml
[registries.nitro]
index = "sparse+https://repo.example.com/repositories/local/crates/index/"
```

The name on the left — `nitro` here — is what you pass to `--registry` and write in
`publish = [...]`. It is local to your machine and does not have to match the repository's name.

## Getting a token

Cargo has no login handshake with the registry; it only stores a token you give it. Mint one under
**Profile → Tokens**, scoped to the repository, then:

```sh
cargo login --registry nitro
# paste the token when prompted
```

That writes `~/.cargo/credentials.toml`:

```toml
[registries.nitro]
token = "…"
```

A token needs `Read` for downloads and `Write` to publish or yank.

## How the token reaches the registry

Cargo sends it as a bare header — no scheme:

```text
Authorization: <token>
```

That is unusual (most clients send `Bearer <token>`), and Nitro Repo accepts it: a value with no
space in it is treated as a token. `Bearer <token>` works too, so the same credential can be used
by `curl` or a script without a second header format.

## CI

Put the token in the environment rather than a file. Cargo reads a per-registry variable:

```sh
export CARGO_REGISTRIES_NITRO_INDEX="sparse+https://repo.example.com/repositories/local/crates/index/"
export CARGO_REGISTRIES_NITRO_TOKEN="$NITRO_TOKEN"

cargo publish --registry nitro
```

The variable name is the registry name uppercased, with `-` becoming `_`.

## Private registries

A private repository sets `auth-required: true` in its `config.json`, which is what tells Cargo to
send a token on _every_ request rather than only on publish. Without that flag Cargo would fetch the
index anonymously, get a `401`, and report a hard failure instead of retrying with credentials.

Nothing has to be configured for this — the flag follows the repository's visibility.

## Next

- [Publishing](/repositories/cargo/publishing/)
- [API tokens and scopes](/admin/tokens/)
