---
title: Query language
description: The grammar behind search — fields, operators, globs and combinators.
sidebar:
  order: 3
---

Search accepts either a plain term or a query in a small language over projects and versions. This
is the language.

```text
repo == releases and scope == dev.kingtux and version ~= "1.*"
name ~= *.jar and not release_type == Snapshot
(scope == dev.kingtux or scope == com.example) and created > 2024-01-01
```

It lives in its own crate with no dependency on the web framework or the database driver, so it can
be lifted out and used elsewhere.

## Shape

A query is a boolean expression over comparisons:

```text
field <operator> value
```

Comparisons combine with `and`, `or`, `not` and parentheses. Two comparisons side by side mean
`and`, so these are the same query:

```text
scope == dev.kingtux and version ~= 1.*
scope == dev.kingtux version ~= 1.*
```

## Fields

| Field          | Aliases                  | What                                                                  |
| -------------- | ------------------------ | --------------------------------------------------------------------- |
| `repository`   | `repo`                   | The repository's name.                                                |
| `storage`      |                          | The storage it lives in.                                              |
| `project`      | `project_key`, `key`     | The full key: `dev.kingtux:tms`, `@nitro/example`.                    |
| `scope`        | `group`, `groupid`       | A Maven `groupId`, an npm scope.                                      |
| `name`         | `artifact`, `artifactid` | The project's name, without its scope.                                |
| `description`  |                          | The project's description.                                            |
| `version`      |                          | A version string.                                                     |
| `release_type` | `releasetype`, `type`    | `Stable`, `Snapshot`, `Alpha`, `Beta`, `ReleaseCandidate`, `Unknown`. |
| `created`      | `created_at`             | When a version was first published.                                   |
| `updated`      | `updated_at`             | When a version was last updated.                                      |

Field names are case-insensitive, and the aliases exist because the same concept is called
different things by different ecosystems — a Maven user should not have to know that this codebase
settled on "scope".

`GET /api/search/fields` returns the list, so a client can offer them rather than hardcoding a
second copy.

## Operators

| Operator | Meaning                 | Applies to           |
| -------- | ----------------------- | -------------------- |
| `==`     | Equals                  | Any field            |
| `!=`     | Does not equal          | Any field            |
| `~=`     | Glob match              | Any field            |
| `!~`     | Does not match the glob | Any field            |
| `>`      | After                   | `created`, `updated` |
| `>=`     | At or after             | `created`, `updated` |
| `<`      | Before                  | `created`, `updated` |
| `<=`     | At or before            | `created`, `updated` |

Text comparisons are **case-insensitive**, matching how coordinates are treated everywhere else.

Ordering comparisons are **rejected on non-temporal fields**. `name > b` would compile to a
lexicographic comparison, which is almost never what anyone means and never says so. You get a
parse error instead.

## Globs

`~=` and `!~` take a glob:

- `*` — any run of characters, including none.
- `?` — exactly one character.

```text
name ~= *-api            every artifact whose name ends in -api
version ~= 1.*           every 1.x version
project ~= @nitro/*      every package in the @nitro scope
```

## Values

A bare word works when it has no spaces, no operators and does not start with a digit:

```text
scope == dev.kingtux
release_type == Snapshot
```

Quote anything else:

```text
name == "my package"
version ~= "1.*"
description ~= "*deprecated*"
```

Quoting is also how you write a value that starts with a digit without it being read as a number.

Dates are written `YYYY-MM-DD` and compared as timestamps:

```text
created > 2024-01-01
updated >= 2025-06-01 and updated < 2025-07-01
```

## Combining

```text
and    both
or     either
not    negate what follows
( )    group
```

`not` binds tighter than `and`, which binds tighter than `or`. Parenthesise when in doubt:

```text
(scope == dev.kingtux or scope == com.example) and created > 2024-01-01
```

Without the parentheses that reads as `scope == dev.kingtux or (scope == com.example and created >
2024-01-01)`, which is a different question.

## Examples

```text
# Everything in one repository
repo == releases

# One group, stable versions only
scope == dev.kingtux and not release_type == Snapshot

# Published this year
created >= 2025-01-01

# API artifacts across two groups
name ~= *-api and (scope == dev.kingtux or scope == com.example)

# A specific package's prereleases
project == @nitro/example and version ~= "*-beta.*"

# Everything but one storage
not storage == archive
```

## Errors

A parse failure comes back as a `400` with the message and the character range it happened at, so a
UI can underline the offending token rather than reporting "syntax error" for the whole string.

```json
{
  "error": "Unknown field `nmae`",
  "start": 0,
  "end": 4
}
```

## Safety

A compiled query is a SQL predicate containing only placeholders, plus the values to bind, so text
from a query never becomes part of a statement. Column names come from the field enum rather than
from the query, so there is nothing to escape there either.
