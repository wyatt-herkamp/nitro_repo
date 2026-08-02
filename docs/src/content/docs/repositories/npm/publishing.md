---
title: Publishing
description: publish, unpublish, deprecate and dist-tags against a Nitro Repo registry.
sidebar:
  order: 3
---

## Publishing

```sh
npm publish
```

With the registry configured in `.npmrc` — either globally or scoped — and a token in place. See
[Authenticating](/repositories/npm/authenticating/).

Publishing requires a `Write` action on the repository.

### What is checked

**The tarball must match its own `integrity`.** npm computes an integrity hash and puts it in the
manifest it sends; Nitro Repo recomputes it from the attachment and refuses a mismatch. That is a
corrupted upload, and storing it would mean every later install fails a check that the registry
could have caught once.

**A version cannot be republished.** A second `npm publish` of a version that already exists is a
`409`. Immutable published versions are the whole contract of a registry — anything that resolved
`1.0.0` yesterday must get the same bytes today. Bump the version, or unpublish first.

**The package name must be legal.** The rules are on the [overview](/repositories/npm/#package-names).

## dist-tags

```sh
npm dist-tag add @nitro/example@2.0.0-beta.1 next
npm dist-tag ls @nitro/example
npm dist-tag rm @nitro/example next
```

`latest` is set automatically to the version being published unless the manifest says otherwise.
Everything else is yours to manage.

The underlying routes, if you are scripting against them:

```text
GET    /-/package/{pkg}/dist-tags
PUT    /-/package/{pkg}/dist-tags/{tag}
DELETE /-/package/{pkg}/dist-tags/{tag}
```

`{pkg}` may be scoped, so the path can contain a `/` inside the package name.

## Unpublishing

```sh
npm unpublish @nitro/example@1.0.0     # one version
npm unpublish @nitro/example --force   # the whole package
```

Removing a version deletes its tarball and its row. Any dist-tag left pointing at it is dropped as
well — a tag aimed at a version that no longer exists turns every install into a 404 on a tarball
that was deleted on purpose.

Removing the last version of a package removes the package, which is what npm does.

npm appends a `/-rev/{revision}` to the request. Nitro Repo strips it: there is no
optimistic-concurrency model here to check it against, so treating it as advisory is honest, where
pretending to validate it would not be.

Unpublishing requires `Write`. There is no separate "may delete" permission.

## Deprecating

```sh
npm deprecate @nitro/example@"<1.0.0" "Use 1.x — 0.x is unmaintained"
```

Marks matching versions deprecated. `npm install` then prints the message. Passing an empty string
clears it.

Deprecating is almost always better than unpublishing. It warns everyone who installs the version
without breaking anyone who has already pinned it.

## Commands that are refused

`npm access`, `npm owner` and `npm star` are recognised and refused with a message npm shows you.
There is no per-package ownership model behind them yet.

That is a deliberate improvement over what happened before, which was a bare `400 Invalid command`
for every one of them — including ones that did work — because only `publish` was recognised at
all.

## Searching

```sh
npm search --registry=https://repo.example.com/repositories/local/npm/ example
```

Backed by the same index as the web UI's search. See [Search](/admin/search/).

## Installing

```sh
npm install @nitro/example
```

Nothing registry-specific. A tarball lives at:

```text
{base}/{package}/-/{unscoped-name}-{version}.tgz
```

Note that the file name uses the **unscoped** name even for a scoped package —
`@nitro/example@1.0.0` is at `@nitro/example/-/example-1.0.0.tgz`. That is npm's convention, not a
Nitro Repo quirk, and it is worth knowing if you are ever constructing these by hand.
