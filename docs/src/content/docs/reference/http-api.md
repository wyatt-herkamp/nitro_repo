---
title: HTTP API
description: The OpenAPI document, authentication schemes, and a map of the routes.
sidebar:
  order: 4
---

Everything the web frontend does, it does through this API, so anything the UI can do is scriptable.

## The OpenAPI document

The document is the reference. This page is a map.

| Route               | What                                               |
| ------------------- | -------------------------------------------------- |
| `/scalar`           | Interactive documentation, browsable and callable. |
| `/open-api-doc-raw` | The raw JSON, for a client generator.              |

Both need `web_server.open_api_routes = true`, which is the default. Turn it off if you would
rather not publish a description of your API surface.

You can also produce the document without a running server, which is how the frontend's TypeScript
types are generated:

```sh
nitro_repo export open-api openapi.json
npx openapi-typescript openapi.json -o src/types/api.d.ts
```

## Authenticating

| Scheme       | Header                             | Notes                                                  |
| ------------ | ---------------------------------- | ------------------------------------------------------ |
| Bearer token | `Authorization: Bearer {token}`    | What scripts should use.                               |
| Session      | `Authorization: session {id}`      | For a client that logged in rather than using a token. |
| Basic        | `Authorization: Basic base64(u:p)` | Password or token in the password field.               |
| Cookie       | set at login                       | The browser.                                           |

```sh
curl -H "Authorization: Bearer $NITRO_TOKEN" https://repo.example.com/api/user/me
```

Some routes require a **session specifically** and refuse a token whatever its scopes: creating a
token, changing a password, and revoking all tokens. A leaked token should not be able to mint more
tokens or lock its owner out.

## Route map

### Instance

| Method | Path               | What                                             |
| ------ | ------------------ | ------------------------------------------------ |
| `GET`  | `/api/info`        | Instance name, description, install state.       |
| `GET`  | `/api/info/scopes` | Every scope, with its title and requirements.    |
| `POST` | `/api/install`     | Create the first administrator. `404` once used. |

### Your account

| Method | Path                        | What                          |
| ------ | --------------------------- | ----------------------------- |
| `POST` | `/api/user/login`           | Sign in, returning a session. |
| `POST` | `/api/user/logout`          | End the session.              |
| `GET`  | `/api/user/me`              | The current user.             |
| `GET`  | `/api/user/me/permissions`  | Their permissions.            |
| `GET`  | `/api/user/whoami`          | A lighter identity check.     |
| `GET`  | `/api/user/sessions`        | Active sessions.              |
| `POST` | `/api/user/change-password` | Session required.             |

### Tokens

| Method   | Path                          | What                                |
| -------- | ----------------------------- | ----------------------------------- |
| `POST`   | `/api/user/token/create`      | Session required.                   |
| `GET`    | `/api/user/token/list`        | Your tokens, without their values.  |
| `GET`    | `/api/user/token/get/{id}`    | One token's metadata.               |
| `DELETE` | `/api/user/token/delete/{id}` | Revoke one.                         |
| `DELETE` | `/api/user/token/revoke-all`  | Revoke every one. Session required. |

### Password reset

| Method | Path                                     | What                       |
| ------ | ---------------------------------------- | -------------------------- |
| `POST` | `/api/user/password-reset/request`       | Send a reset email.        |
| `GET`  | `/api/user/password-reset/check/{token}` | Is this token still valid. |
| `POST` | `/api/user/password-reset/{token}`       | Set the new password.      |

### User management

Needs `user_manager`.

| Method   | Path                                           |
| -------- | ---------------------------------------------- |
| `GET`    | `/api/user-management/list`                    |
| `GET`    | `/api/user-management/get/{id}`                |
| `GET`    | `/api/user-management/get/{id}/permissions`    |
| `POST`   | `/api/user-management/create`                  |
| `POST`   | `/api/user-management/is-taken`                |
| `PUT`    | `/api/user-management/update/{id}`             |
| `PUT`    | `/api/user-management/update/{id}/permissions` |
| `PUT`    | `/api/user-management/update/{id}/password`    |
| `DELETE` | `/api/user-management/{id}/tokens/revoke-all`  |

### Storages

| Method | Path                      | What                                       |
| ------ | ------------------------- | ------------------------------------------ |
| `GET`  | `/api/storage/list`       | Every storage.                             |
| `POST` | `/api/storage/new/{type}` | Create one. `Local`, `FileSystemV2`, `S3`. |
| `GET`  | `/api/storage/{id}`       | One storage.                               |
| `PUT`  | `/api/storage/{id}`       | Rename it, or change its config.           |
| `GET`  | `/api/storage/s3/regions` | The named AWS regions, for the UI.         |

The config is validated against the backend before it is saved, so a create or update that succeeds
is a storage that works.

There is no delete route.

### Repositories

| Method | Path                | What                           |
| ------ | ------------------- | ------------------------------ |
| `GET`  | `/api/repository/…` | List, read, create, configure. |

Repository creation takes a `configs` map alongside the name and type — the required configs for
that type have to be supplied at creation. Visibility is set separately, afterwards.

### Projects and search

| Method | Path                 | What                                      |
| ------ | -------------------- | ----------------------------------------- |
| `GET`  | `/api/project/…`     | Project and version metadata.             |
| `GET`  | `/api/search`        | `query` or `text`, plus `limit`/`offset`. |
| `GET`  | `/api/search/fields` | Queryable fields, for a client's UI.      |

See [Query language](/reference/query-language/).

### npm login

| Method | Path                       | What                            |
| ------ | -------------------------- | ------------------------------- |
| `GET`  | `/api/npm/login/{session}` | What is this login session for. |
| `POST` | `/api/npm/login/{session}` | Approve it and mint the token.  |

These back the browser login page; npm does not call them directly. See
[npm → Authenticating](/repositories/npm/authenticating/).

## Not under `/api`

| Path                                     | What                                         |
| ---------------------------------------- | -------------------------------------------- |
| `/repositories/{storage}/{repository}/…` | The repositories themselves — Maven and npm. |
| `/badge/{storage}/{repository}/…`        | Generated SVG badges.                        |

The repository routes are not a REST API; they are whatever protocol the repository type speaks.

## Errors

Errors are JSON with a `message`, and `error` and `details` where there is something useful to say:

```json
{
  "message": "Repository not found",
  "error": null,
  "details": null
}
```

| Status | Usually means                                                            |
| ------ | ------------------------------------------------------------------------ |
| `400`  | Malformed request, or a push rule refused it.                            |
| `401`  | No credentials, or bad ones.                                             |
| `403`  | Authenticated, but not permitted.                                        |
| `404`  | Not found — or hidden from you, which is deliberately indistinguishable. |
| `409`  | A conflict: a name taken, a version already published.                   |
| `422`  | The body did not match the schema.                                       |

## CORS

The API allows any origin. That is fine for reads of public repositories and deliberate for
browser-based tooling; it is worth knowing before you rely on the browser's same-origin policy for
anything here. Authentication is by header or cookie, and permissions are enforced server-side.
