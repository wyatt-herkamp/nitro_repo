---
title: Users and permissions
description: Accounts, the four instance-wide permissions, per-repository access, and how a request is authenticated.
sidebar:
  order: 2
---

## Accounts

The first account is created once, at `/admin/install`, and is an administrator. Every account
after that comes from **Admin → Users → Create**, which needs the `user_manager` permission.

A user has a username, an email, an optional display name, and a password checked against
`security.password_rules`.

## Instance-wide permissions

Four flags, each independent of the others:

| Permission                   | What it allows                                                                  |
| ---------------------------- | ------------------------------------------------------------------------------- |
| `admin`                      | Everything, including instance settings.                                        |
| `user_manager`               | List, create, edit and delete users, and change their permissions.              |
| `system_manager`             | Create and delete repositories and storages, and read/write across all of them. |
| `default_repository_actions` | What this user may do in a repository they have no explicit grant for.          |

`default_repository_actions` is the one that decides most day-to-day behaviour. It is a set drawn
from `Read`, `Write` and `Edit`:

- **`Read`** — resolve and download from a repository.
- **`Write`** — deploy and publish to it.
- **`Edit`** — change its settings and configs.

A user with `[Read, Write]` as their defaults can publish to any repository unless something says
otherwise. A user with `[]` can reach only the repositories they were explicitly granted.

## Per-repository permissions

Granting per repository, on a user's page, overrides the defaults for that repository. This is what
you want for the common shapes:

- **A CI account for one project.** No default actions, `Write` on the one repository it deploys
  to.
- **A contractor.** `Read` by default, no access to `internal`.
- **A team lead.** `Read` and `Write` by default, plus `Edit` on the repositories they own.

## Visibility versus permission

The two are separate and it is worth being precise about the difference.

**Visibility** is a property of the repository and decides what happens for an _unauthenticated_
request:

| Visibility | An anonymous read                                     |
| ---------- | ----------------------------------------------------- |
| `Public`   | Allowed.                                              |
| `Hidden`   | Allowed, but the repository is not listed or indexed. |
| `Private`  | Refused — the client is asked to authenticate.        |

**Permission** decides what happens for an authenticated one. Writes always require authentication
and a `Write` action, whatever the visibility. A public repository is public to read, never to
publish to.

A repository can also be marked **inactive**, which trumps both: an inactive repository refuses
every request from everyone. It is the way to retire a repository without deleting what is in it.

## How a request is authenticated

Four mechanisms, in the order you are likely to meet them:

| Mechanism      | Header                             | Used by                                               |
| -------------- | ---------------------------------- | ----------------------------------------------------- |
| Session cookie | set at login                       | The web frontend.                                     |
| Session header | `Authorization: session {id}`      | API clients that logged in rather than using a token. |
| API token      | `Authorization: Bearer {token}`    | CI, scripts, npm.                                     |
| Basic          | `Authorization: Basic base64(u:p)` | Maven, and npm's legacy login.                        |

Basic auth accepts either a password or an API token in the password field. Maven has no other way
to send credentials, so putting a token there is how you avoid handing your `settings.xml` a real
password. A repository can require this with the `must_use_auth_token_for_push` push rule.

Some routes require a **session specifically** and will refuse a token no matter its scopes:
changing a password, creating a token, and revoking all tokens. That is deliberate — a leaked token
should not be able to mint more tokens or lock its owner out.

## Password resets

`POST /api/user/password-reset/request` sends a reset link, which needs `[email]` to be configured.
Without it the endpoint has nowhere to send anything.

An administrator can also set a password directly from the user's page, which needs no email.
