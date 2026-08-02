---
title: Docker
description: A hosted OCI container registry — how it is addressed, what is supported, and what is not there yet.
sidebar:
  order: 1
---

A Docker repository is a hosted container registry speaking the
[OCI distribution API](https://github.com/opencontainers/distribution-spec/blob/main/spec.md) —
the one `docker`, `podman`, `containerd`, BuildKit, `crane` and `skopeo` all use.

## Addressing

Docker is the one repository type that is **not** reached under
`/repositories/{storage}/{repository}`. A client always requests `/v2/...` at the root of whatever
host it was given, and there is no setting that adds a path prefix. So there are two ways to address
a registry, and you can use either.

### With a hostname

Attach a [custom domain](/admin/custom-domains/) to the repository, and image names are bare:

```sh
docker pull docker.example.com/myimage:1.0
```

This is the form to prefer. It is what every published image reference looks like, and it is the
only one that produces short names.

### Without one

The first two segments of the image name are the storage and the repository:

```sh
docker pull repo.example.com/local/docker/myimage:1.0
```

Nothing to configure — this works on the instance's own host as soon as the repository exists. The
cost is verbose image names.

:::caution[Docker requires TLS]
A Docker daemon refuses a plain-HTTP registry unless it is `localhost`, or the host is listed in
`insecure-registries` in `/etc/docker/daemon.json`. Put the instance behind TLS.
:::

## What works

|                       |                                                                                  |
| --------------------- | -------------------------------------------------------------------------------- |
| **Push and pull**     | `docker push` / `docker pull`, and the same over podman, containerd, crane.      |
| **Resumable uploads** | `POST` / `PATCH` / `PUT`, monolithic or chunked.                                 |
| **Multi-arch**        | Image indexes, with their per-platform children.                                 |
| **Digest pulls**      | `image@sha256:…`, byte-identical to what was pushed.                             |
| **`HEAD` checks**     | So a push skips layers already present instead of re-uploading them.             |
| **Tags and catalog**  | `/tags/list` and `/_catalog`, with `?n=` / `?last=` paging.                      |
| **Referrers**         | The OCI referrers API, for signatures and SBOMs attached to an image.            |
| **Deletes**           | By tag, and by manifest digest.                                                  |
| **Auth**              | The `WWW-Authenticate: Bearer` challenge and token endpoint `docker login` uses. |
| **Shared layers**     | Blobs are content-addressed, so images in one repository share their bases.      |

## What is not there

Stated plainly, because each of these is something a registry is often assumed to do:

- **No pull-through cache.** There is no `Proxy` mode, so a repository cannot mirror Docker Hub.
- **No garbage collection.** Deleting a tag or a manifest does not reclaim the blobs behind it.
  Nothing reference-counts layers, so storage only grows.
- **No blob deletes.** `DELETE` on a blob answers `405`. Removing a layer that another manifest
  shares would break images that were fine a moment ago.
- **No cross-repository mounting.** A `?mount=` request falls back to a normal upload, which the
  spec allows. The client then pushes the blob rather than borrowing it.
- **No `Range` requests on blobs.** A ranged `GET` is answered in full. Clients cope; a resumed
  interrupted pull starts over.
- **Uploads are node-local.** An in-progress push is buffered to the server's staging directory, so
  it is bound to the node that started it and to that node's disk. Behind a load balancer, pin
  upload sessions to one backend. See [Configuration](/repositories/docker/configuration/#staging).

## Image names

The registry enforces the distribution spec's grammar: lowercase `a-z`, `0-9`, and `.`, `_`, `-`
between alphanumerics, in `/`-separated components. `library/alpine` and `team/project/service` are
fine; `Alpine` is not — Docker itself refuses to push an uppercase name, so accepting one would
create an image nobody could pull back.

In prefix mode the storage and repository count as the first two components, and the rest is the
image name proper.

## How storage is laid out

Per repository:

```text
blobs/sha256/{hex}      layers and image configs, content-addressed
manifests/sha256/{hex}  manifests, stored as the exact bytes pushed
```

A manifest is re-served byte-for-byte, never re-serialised — its digest is over those bytes, so
rewriting them would change the image's identity and break every client that pinned it.

Tags are versions of a project, which is what gives a Docker image the same project pages, badges
and search as every other repository type.

## Next

- [Authenticating](/repositories/docker/authenticating/) — `docker login`, tokens, CI.
- [Pushing and pulling](/repositories/docker/pushing/)
- [Configuration](/repositories/docker/configuration/)
