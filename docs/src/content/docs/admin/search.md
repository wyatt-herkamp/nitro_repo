---
title: Search
description: Finding artifacts by text or by query, and what search can and cannot see.
sidebar:
  order: 4
---

Search runs over **projects and versions** — the rows that exist because something was deployed —
not over the files themselves. So you can find `dev.kingtux:tms` version `1.1.0`; you cannot search
for a class inside its jar.

## From the UI

The search page takes either a plain term or a query. A plain term is matched against the project
key, the name and the description, case-insensitively. That is the right default and covers most of
what people want.

Switch to the query syntax when you need something the box cannot express: everything published
since a date, everything but snapshots, one scope across several repositories.

```text
scope == dev.kingtux and version ~= 1.*
name ~= *-api and not release_type == Snapshot
created > 2025-01-01 and repo == releases
```

The full grammar is in [Query language](/reference/query-language/).

## From the API

```sh
# plain text
curl "https://repo.example.com/api/search?text=tms&limit=25"

# a query
curl --get "https://repo.example.com/api/search" \
     --data-urlencode 'query=scope == dev.kingtux and version ~= 1.*'
```

| Parameter | What                                                       |
| --------- | ---------------------------------------------------------- |
| `query`   | A query in the artifact query language.                    |
| `text`    | A plain term. Used when `query` is absent.                 |
| `limit`   | How many results to return. Defaults to 50, capped at 200. |
| `offset`  | Where to start, for paging.                                |

With neither `query` nor `text`, everything the caller can read comes back, newest first. Results
are always ordered by when a version was last updated.

The response carries the results, how many were returned, and the query that actually ran — a
plain `text` search is expanded into a query, and the expansion is echoed back so you can see what
was searched for.

```json
{
  "results": [
    {
      "project_key": "dev.kingtux:tms",
      "name": "tms",
      "scope": "dev.kingtux",
      "version": "1.1.0",
      "release_type": "Stable",
      "repository": "releases",
      "storage": "local",
      "version_path": "dev/kingtux/tms/1.1.0",
      "created_at": "2025-06-01T12:00:00Z",
      "updated_at": "2025-06-01T12:00:00Z"
    }
  ],
  "count": 1,
  "query": "scope == dev.kingtux and version ~= \"1.*\""
}
```

`count` is how many rows came back, not how many matched. Counting every match costs a second scan
of the table and nothing needs the number, so it is not paid for.

`GET /api/search/fields` lists the queryable fields and whether each accepts ordering comparisons,
so a client can offer them rather than hardcoding its own list.

## What search can see

**Results are filtered by who is asking.** Anything in a repository that is not `Public` is dropped
unless the caller has `Read` on it. Both `Hidden` and `Private` behave this way, so an anonymous
search cannot be used to enumerate a repository it could not fetch from — which is the failure mode
worth caring about, since a listing of every internal coordinate is useful to an attacker even
without the artifacts.

**Proxy repositories only appear once something has been fetched through them.** A proxy has no
inventory of its upstream — it discovers artifacts by being asked for them — so a freshly created
proxy repository is invisible to search, and grows into it as builds pull through it.

**Versions are indexed when they are published, not when files are written.** For Maven that means
the POM: uploading a jar with no POM stores the file but creates no project row, so it is not
searchable. This is usually a sign the deploy did not go the way you thought.
