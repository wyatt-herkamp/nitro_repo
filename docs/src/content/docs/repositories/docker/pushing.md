---
title: Pushing and pulling
description: docker push, multi-arch images, tags and deletes.
sidebar:
  order: 3
---

## Pushing

```sh
docker tag myimage:1.0 repo.example.com/local/docker/myimage:1.0
docker push repo.example.com/local/docker/myimage:1.0
```

With a hostname on the repository, drop the `local/docker/` prefix.

### What happens

For each layer and for the image config, the client:

1. `HEAD`s the blob to see whether it is already here — if so, nothing transfers;
2. `POST`s to open an upload and gets a `Location` back;
3. `PATCH`es the bytes into it, in one go or in chunks;
4. `PUT`s with `?digest=sha256:…` to commit.

The registry hashes what it received and compares. A mismatch is a `400 DIGEST_INVALID`, the session
is discarded, and nothing reaches storage.

Then the manifest is `PUT`. Before it is accepted, every blob it references must already be present
— otherwise a push would succeed and every pull of it would fail on a missing layer, with nothing to
say which push was at fault.

## Pulling

```sh
docker pull repo.example.com/local/docker/myimage:1.0
docker pull repo.example.com/local/docker/myimage@sha256:2c009b…
```

A digest pull returns exactly the bytes that were pushed.

## Tags

A tag is a mutable pointer to a manifest, and moving it is normal — `:latest` moves with every
push. Re-tagging is _not_ refused the way re-publishing a version is in Maven, npm or Cargo; a
container tag is not a version.

```sh
curl https://repo.example.com/v2/local/docker/myimage/tags/list
```

```json
{ "name": "local/docker/myimage", "tags": ["1.0", "1.1", "latest"] }
```

Tags come back in lexical order, and page with `?n=` and `?last=`.

The whole registry's image list:

```sh
curl https://repo.example.com/v2/local/docker/_catalog
```

## Multi-arch images

Push each platform's manifest by digest, then the index that names them:

```sh
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --tag repo.example.com/local/docker/myimage:1.0 \
  --push .
```

The index is refused until all of its children are present, for the same reason a manifest is
refused until its blobs are. The per-platform manifests are reachable by digest but are not tags —
only the index gets one.

## Deleting

```sh
# Remove a tag. The manifest stays, reachable by digest.
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  https://repo.example.com/v2/local/docker/myimage/manifests/1.0

# Remove the manifest itself, and any tag still pointing at it.
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  https://repo.example.com/v2/local/docker/myimage/manifests/sha256:2c009b…
```

Deleting a manifest also drops any tag aimed at it — a tag left pointing at bytes that are gone
turns every pull into a `404` on something the tag list still advertises.

:::caution[Blobs are never removed]
Layers are content-addressed and nothing reference-counts them, so deleting a manifest does not
reclaim its storage, and `DELETE` on a blob answers `405`. Removing a layer another image shares
would break that image. Storage only grows; plan for it.
:::

## Referrers

Signatures, SBOMs and other artifacts attached to an image are found through the referrers API:

```sh
curl https://repo.example.com/v2/local/docker/myimage/referrers/sha256:2c009b…
```

An unknown subject answers `200` with an empty index rather than `404` — a client uses this to
discover whether anything exists at all.

## Next

- [Configuration](/repositories/docker/configuration/)
