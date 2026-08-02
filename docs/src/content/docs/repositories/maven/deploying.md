---
title: Deploying and resolving
description: Client configuration for Maven and Gradle, snapshots, checksums, and the protocol underneath.
sidebar:
  order: 2
---

## Resolving

Add the repository to your build. This is all it takes for a public one:

```xml
<repositories>
  <repository>
    <id>nitro</id>
    <url>https://repo.example.com/repositories/local/releases</url>
  </repository>
</repositories>
```

```kotlin
repositories {
    maven { url = uri("https://repo.example.com/repositories/local/releases") }
}
```

For a private repository, add credentials — see [Authentication](#authentication) below.

Snapshots need to be enabled explicitly in Maven, because they are off by default in a
`<repository>` block:

```xml
<repository>
  <id>nitro-snapshots</id>
  <url>https://repo.example.com/repositories/local/snapshots</url>
  <releases><enabled>false</enabled></releases>
  <snapshots><enabled>true</enabled></snapshots>
</repository>
```

## Deploying

```xml
<distributionManagement>
  <repository>
    <id>nitro-releases</id>
    <url>https://repo.example.com/repositories/local/releases</url>
  </repository>
  <snapshotRepository>
    <id>nitro-snapshots</id>
    <url>https://repo.example.com/repositories/local/snapshots</url>
  </snapshotRepository>
</distributionManagement>
```

```kotlin
publishing {
    repositories {
        maven {
            url = uri("https://repo.example.com/repositories/local/releases")
            credentials {
                username = System.getenv("NITRO_USER")
                password = System.getenv("NITRO_TOKEN")
            }
        }
    }
}
```

Then `mvn deploy` or `gradle publish`.

## Authentication

Maven only speaks HTTP Basic, so credentials go in `~/.m2/settings.xml`. The `<id>` must match the
repository `<id>` in your POM:

```xml
<servers>
  <server>
    <id>nitro-releases</id>
    <username>ci</username>
    <password>${env.NITRO_TOKEN}</password>
  </server>
</servers>
```

**Put an API token in the password field, not your password.** Nitro Repo accepts either, but a
token can be scoped down to "publish to this one repository" and revoked on its own. See
[API tokens and scopes](/admin/tokens/).

A repository can insist on this with the `must_use_auth_token_for_push` rule, after which a real
password is refused for deploys.

## Snapshots

Deploy a `-SNAPSHOT` version and Maven uploads timestamped files rather than the bare name:

```text
tms-1.0.0-20250731.142233-1.jar
tms-1.0.0-20250731.142233-1.pom
```

Nitro Repo generates the per-version metadata that maps `1.0.0-SNAPSHOT` onto the newest of those:

```text
{group}/{artifact}/1.0.0-SNAPSHOT/maven-metadata.xml
```

That document is built by listing the version directory, so it always reflects what is actually
stored. There is no state to fall out of date and nothing to rebuild.

There is **no snapshot retention policy** — old builds accumulate until something removes them. A
snapshot repository that a CI job deploys to on every commit will grow without bound. Plan for
that.

## Checksums

Maven uploads `.md5` and `.sha1` beside every artifact and expects to fetch them back. Both
directions work, and both do something more useful than storing a blob:

**On upload**, a checksum is compared against the artifact it names. A mismatch is refused, which
catches a corrupted transfer at the point it happened rather than in someone's build a week later.
A checksum for a file that was never uploaded is accepted rather than refused — Maven's upload
order makes that a client quirk, not a corruption.

**On download**, a checksum that was never uploaded is generated from the stored artifact rather
than 404'd. `md5`, `sha1`, `sha256` and `sha512` all work, whether or not the client sent them.

`md5`, `sha1` and `sha256` come from hashes the storage layer already recorded when the file was
written, so they cost nothing. `sha512` is not among those, so serving one reads the artifact back.

Checksums of `maven-metadata.xml` are derived from the same bytes about to be served, so they
cannot disagree with the document.

## What a deploy is checked against

In order, before anything is written:

1. **Push policy** — a `Release` repository refuses `-SNAPSHOT`, a `Snapshot` repository refuses
   releases, `Mixed` accepts both.
2. **Semver**, if `require_semver` is on. `1.0.0-SNAPSHOT` is a legal semver prerelease, so
   snapshots pass.
3. **Overwrite**, if `allow_overwrite` is off. Files Maven legitimately rewrites — metadata,
   checksums — are exempt.
4. **Project membership**, if `must_be_project_member` is on. Only applies once the project exists;
   whoever pushes first creates it and becomes an owner.
5. **POM against path**, for a `.pom`. A POM declaring `dev.kingtux:tms:1.0.0` cannot be deployed
   to `com/example/other/9.9.9/`.

A rejection is a `400` with a message naming the rule. See
[Configuration](/repositories/maven/configuration/).

## Registration

Deploying a POM is what creates the project and version rows behind browse, search and badges. The
name, description and license come from the POM.

This means a jar deployed without its POM is stored and served, but is invisible to everything
except a direct `GET`. If artifacts are missing from search, that is the first thing to check.

A registration failure fails the deploy. It used to be logged and swallowed, so a deploy could
report `201` while leaving the artifact invisible with nothing to explain why.

## The protocol

Nothing here is unusual — it is the standard Maven layout — but for reference:

|                    |                                                                           |
| ------------------ | ------------------------------------------------------------------------- |
| Deploy             | `PUT {base}/{group as path}/{artifact}/{version}/{file}`                  |
| Resolve            | `GET` the same path                                                       |
| Coordinates        | `dev.kingtux:tms:1.0.0` → `dev/kingtux/tms/1.0.0/`                        |
| Auth               | `Authorization: Basic base64(user:password-or-token)`                     |
| Private repository | Answers `401` with `WWW-Authenticate`, and Maven retries with credentials |

A `GET` for a directory renders an HTML index, so a repository can be browsed in a browser as well
as by a build tool.
