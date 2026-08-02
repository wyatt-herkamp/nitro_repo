---
title: Storages
description: Local, FileSystemV2 and S3 — how each stores artifacts, and which to pick.
sidebar:
  order: 1
---

A storage is a place bytes go. Repositories live inside one, and every artifact a repository holds
is written through it.

Storages are configured at runtime — the settings live in Postgres, not in `nitro_repo.toml` — so
adding one is a form in **Admin → Storages → Create**, not a restart. Three types ship.

## Choosing

|                 | Local                              | FileSystemV2          | S3                         |
| --------------- | ---------------------------------- | --------------------- | -------------------------- |
| Where           | Disk                               | Disk                  | An S3 bucket               |
| Layout          | Files and directories you can `ls` | One object per file   | Objects under a key prefix |
| Metadata        | `.nr-meta` sidecar files           | Inside the object     | `.nr-meta` sidecar objects |
| Range requests  | No                                 | Yes                   | Yes                        |
| Compression     | No                                 | Optional zstd or gzip | No                         |
| Needs a network | No                                 | No                    | Yes                        |

**If you are not sure, use Local.** It is the oldest and most boring of the three, the on-disk
layout is exactly what you would guess, and a backup is `tar`.

**Use FileSystemV2** when you want the disk simplicity without the sidecars — see below for why
that matters — or when you are serving large artifacts and want range requests to work.

**Use S3** when the artifacts should not be on the server's disk at all: you want the durability
someone else operates, or the server is ephemeral.

## Local

The original filesystem backend. Files land at:

```text
{path}/{repository-uuid}/{artifact path}
```

Metadata — content type, timestamps, the md5/sha1/sha256/sha3 hashes — goes in a `.nr-meta` file
beside each artifact, encoded with postcard. Those sidecars are maintained by a background task, so
they are eventually consistent: immediately after a write the file is there and correct, and its
metadata may lag by a moment.

### Configuration

| Field  | Required | What                                              |
| ------ | -------- | ------------------------------------------------- |
| `path` | Yes      | Root directory. One sub-directory per repository. |

The path must be writable by the user the server runs as. Under Docker it must be inside a volume;
the default working directory is not persisted.

### The sidecar problem

Sidecars are why FileSystemV2 exists. They can drift from the artifact they describe — a crash
between writing the file and writing its metadata leaves the pair inconsistent — they have to be
filtered out of every directory listing so they do not appear as artifacts, and every metadata
update rewrites a second file. None of that is fatal, and Local is used and works. It is just
avoidable.

## FileSystemV2

Also plain disk, but each artifact is stored as a single self-contained object: a fixed header, its
metadata, its tags, and then the content, with reserved slack so a metadata update does not have to
move the content.

What that buys:

- **No sidecars.** Nothing to filter out of a listing, nothing to drift, no background task.
- **Range requests.** The format can seek within the content, so `Range:` headers work. Local
  cannot do this.
- **Streaming hashes.** Content is written first and the header last, so checksums are computed as
  bytes arrive rather than by reading the finished file back.
- **Optional compression.** POMs, `maven-metadata.xml` and packuments are text and compress well.

Publishing is a write to a temporary file followed by a rename, so a crash mid-upload leaves the
previous version intact rather than a half-written one.

### Configuration

| Field         | Required | Default | What                                              |
| ------------- | -------- | ------- | ------------------------------------------------- |
| `path`        | Yes      | —       | Root directory. One sub-directory per repository. |
| `compression` | No       | `None`  | `None`, `Zstd` (level 3) or `Gzip` (level 6).     |
| `sync`        | No       | `false` | `fsync` each object before publishing it.         |

Compression levels are mid-range on purpose: the point is to shrink text-ish artifacts without
making a deploy wait on the codec. A repository full of already-compressed jars will not gain much.

`sync` costs a flush per write in exchange for a completed upload surviving a power loss. It is off
by default because the rename is already atomic — a crash without it loses the write rather than
corrupting anything.

:::note[No migration from Local]
The two formats are not interchangeable and there is no converter. Moving a repository between them
means re-deploying its contents.
:::

## S3

An S3 bucket, or anything that speaks the S3 API — MinIO, Ceph RGW, Cloudflare R2, Backblaze B2.

Artifacts are objects keyed by repository UUID and path. Metadata goes in `.nr-meta` sidecar
objects using the same encoding Local uses, rather than in object tags or `x-amz-meta-*`: tags cap
at ten entries of 256 characters and user metadata at 2 KB in total, and repository metadata does
not reliably fit in either.

Directory listings are prefix-based — `ListObjectsV2` with `/` as the delimiter — so common
prefixes are directories and keys are files. Reads stream rather than buffering the whole object,
and large uploads go through multipart.

### Configuration

| Field           | Required | Default | What                                                   |
| --------------- | -------- | ------- | ------------------------------------------------------ |
| `bucket_name`   | Yes      | —       | The bucket. It must already exist.                     |
| `region`        | One of   | —       | A named AWS region.                                    |
| `endpoint`      | One of   | —       | A custom endpoint, for anything that is not AWS.       |
| `custom_region` | No       | —       | Region name to sign with when using a custom endpoint. |
| `access_key`    | No       | —       | Explicit access key.                                   |
| `secret_key`    | No       | —       | Explicit secret key.                                   |
| `path_style`    | No       | `true`  | Path-style addressing rather than virtual-hosted.      |

A custom endpoint takes precedence over a named region if both are set.

### Credentials

Leave `access_key` and `secret_key` **both** empty to use the AWS default credential chain —
environment variables, the shared profile, IMDS, web identity. That is how you run against an IAM
role or IRSA on EKS, and it is the better option when it is available, because there is no long-
lived key stored in your database.

Setting one key without the other is a misconfiguration and is rejected, rather than being treated
as a request for the environment.

### MinIO and other S3-compatible servers

```text
endpoint:      http://minio.internal:9000
custom_region: us-east-1
access_key:    …
secret_key:    …
path_style:    true
```

Keep `path_style` on. Virtual-hosted addressing puts the bucket in the hostname, which needs
wildcard DNS that a self-hosted server almost never has.

`custom_region` is only there to satisfy SigV4, which insists on _some_ region in the credential
scope even when the endpoint ignores it. If you leave it empty, `us-east-1` is used, which is what
S3-compatible servers conventionally accept.

## Editing

A storage can be renamed and its configuration edited from **Admin → Storages → _name_**. Changes
are validated against the backend before they are saved — a bucket that cannot be reached, or a
path that cannot be written, is rejected at the point of editing rather than at the next deploy.
The same check runs at creation, so a storage that saves is a storage that works.

Be careful with anything that changes _where_ artifacts are. Repointing a Local storage at a
different path does not move what was already written to the old one; the repositories will simply
look empty.

:::caution[Storages cannot be deleted]
There is no delete endpoint. Once a storage exists it stays, so a mistake has to be corrected by
renaming rather than by starting over — or by removing the row by hand.

Renaming is not free either: the storage name is the first path segment of every repository URL, so
every client pointed at a repository inside it has to be updated. Rename before anyone is using it,
not after.
:::
