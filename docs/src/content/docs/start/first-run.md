---
title: First run
description: Create the first administrator, a storage and a repository, then deploy something into it.
sidebar:
  order: 3
---

A fresh instance has no users, no storages and no repositories. Three steps in order, because each
one needs the previous.

## 1. The first administrator

Open the instance in a browser. With nobody registered, `/admin/install` is the page you want — the
form there creates the first user and grants them administrator.

```text
http://localhost:6742/admin/install
```

This route works exactly once. Once a user exists, the endpoint behind it answers `404`, so it
cannot be used to mint a second administrator later. Make further accounts from
**Admin → Users → Create**.

Passwords are checked against `security.password_rules` — eight characters with an upper case, a
lower case, a digit and a symbol, unless you have changed it. A password that fails the rules is
rejected without much explanation, so if the form refuses a password that looks fine, that is why.

## 2. A storage

**Admin → Storages → Create.** A storage is a place bytes go. Repositories live inside one, so
there has to be one before there can be a repository.

Pick a type:

| Type           | Use it when                                                                         |
| -------------- | ----------------------------------------------------------------------------------- |
| `Local`        | You want files on disk, laid out as ordinary directories you can inspect with `ls`. |
| `FileSystemV2` | You want files on disk, with metadata and content in one object and range requests. |
| `S3`           | The artifacts should live in S3, MinIO, or anything else that speaks the S3 API.    |

For `Local` and `FileSystemV2` the only required field is a path. It must be writable by the user
the server runs as, and it should not be inside the working directory of a container that gets
replaced. If `suggested_local_storage_path` is set in the config, the form pre-fills it.

[Storages](/admin/storages/) covers the trade-offs and the S3 fields in detail.

## 3. A repository

**Admin → Repositories → Create.** A repository has a storage, a name, and a type — `maven` or
`npm`.

The name becomes part of every URL a client uses, and it cannot be changed casually afterwards, so
it is worth a moment's thought. The convention that ages best is one repository per purpose rather
than one per project: `releases`, `snapshots`, `internal`, `central-proxy`.

New repositories are **public** by default — that is the column default in the database, not a
choice the create form makes. If the repository is for internal artifacts, set its visibility
before you deploy into it, not after. It is on the repository's page:

| Visibility | Reads                                                             |
| ---------- | ----------------------------------------------------------------- |
| `Public`   | Anyone, no credentials. The default.                              |
| `Hidden`   | Anyone with the URL, but the repository is not listed or indexed. |
| `Private`  | Only authenticated users with read access.                        |

Writes always require authentication, whatever the visibility.

## 4. Deploy something

The repository's page renders the client snippets for its type — `distributionManagement` and a
`<repository>` block for Maven, an `.npmrc` for npm — already filled in with your instance's URL.
Copy from there rather than from here, and they will be right.

The base URL every client needs is:

```text
{app_url}/repositories/{storage}/{repository}/
```

The trailing slash matters for npm — without it npm drops the last path segment and requests land
in the wrong place.

For Maven, put that URL in your `distributionManagement` and your credentials in
`~/.m2/settings.xml`. For npm, point `.npmrc` at it and run `npm login`. Both are covered properly
in [Maven → Deploying](/repositories/maven/deploying/) and
[npm → Publishing](/repositories/npm/publishing/).

### Or fill it with fixtures

An empty instance is hard to judge — browsing, search, metadata merging and badges all behave
differently with one artifact than with forty. The `seed` subcommand deploys a whole suite over the
real protocols:

```sh
nitro_repo seed --config seed.toml --write-example
$EDITOR seed.toml     # the password, and the storage path
nitro_repo seed --config seed.toml
```

See [Seeding an instance](/admin/seeding/).

## What to do next

- **Make an API token.** Publishing from CI with a user password works, and is a bad idea. Tokens
  carry scopes, so a token that can publish need not also be able to delete repositories. See
  [API tokens and scopes](/admin/tokens/).
- **Decide who can see what.** [Users and permissions](/admin/users/).
- **Set `site.app_url`** if you have not. npm's browser login refuses to run without it.
