---
title: API tokens and scopes
description: Creating tokens, what each scope permits, and how to use one from Maven, npm and curl.
sidebar:
  order: 3
---

An API token authenticates as you without being your password. Use one anywhere a machine needs
access: CI, a deploy script, `npm publish`, a `settings.xml`.

## Creating one

**Profile → Tokens → Create.**

A token has a name, an optional description, a set of scopes, optional per-repository scopes, and
an optional expiry in days. At least one scope is required — a token with none is rejected rather
than created as a token that can do nothing.

**The value is shown once.** It is stored SHA-256 hashed, so there is no way to recover it later.
If you lose it, delete the token and make another.

Creating a token requires a browser session; a token cannot be used to mint another. Neither can it
be used to change a password or to revoke every token. That is deliberate: a leaked token should
not be able to entrench itself or lock its owner out.

## Scopes are ceilings

The rule that makes the rest of this simple:

> A scope is a ceiling, never a grant. Holding one still requires the user behind the token to have
> the underlying permission.

A token carrying `DeleteRepository` owned by a user who cannot delete repositories still cannot
delete anything. This is why the administrative scopes below are safe to offer to everyone, and why
the sensible habit is to mint a token with the _fewest_ scopes that let the job finish — the
ceiling is the only thing that protects you if the token leaks, since the user's permissions are
already whatever they are.

## The scopes

### Repository

| Scope              | Permits                                                              |
| ------------------ | -------------------------------------------------------------------- |
| `ReadRepository`   | Read the repositories the user has access to.                        |
| `WriteRepository`  | Deploy and publish to them.                                          |
| `EditRepository`   | Change their settings.                                               |
| `CreateRepository` | Create new repositories. Needs `system_manager`.                     |
| `DeleteRepository` | Delete repositories, and everything in them. Needs `system_manager`. |

### Storage

| Scope           | Permits                                           |
| --------------- | ------------------------------------------------- |
| `ReadStorage`   | List storages and read their settings.            |
| `ManageStorage` | Create and edit storages. Needs `system_manager`. |

### User

| Scope            | Permits                                                                                     |
| ---------------- | ------------------------------------------------------------------------------------------- |
| `ReadSelf`       | Read your own profile, permissions and sessions.                                            |
| `UpdatePassword` | Change your password — and a session is required as well, so a token alone is never enough. |

### User management

| Scope        | Permits                                  |
| ------------ | ---------------------------------------- |
| `ReadUser`   | List users and read their profiles.      |
| `CreateUser` | Create users.                            |
| `EditUser`   | Edit users, including their permissions. |
| `DeleteUser` | Delete users.                            |

All four need `user_manager`.

### System

| Scope        | Permits                                              |
| ------------ | ---------------------------------------------------- |
| `ReadSystem` | Read instance-wide settings. Needs `system_manager`. |
| `EditSystem` | Change them. Needs `admin`.                          |

## Per-repository scopes

Alongside the instance-wide scopes, a token can carry a set of actions — `Read`, `Write`, `Edit` —
scoped to specific repositories. This is how you mint a token for one pipeline:

- no instance-wide `WriteRepository`,
- `Write` on `releases` only.

Even if the owner can publish everywhere, that token cannot.

## Using a token

### curl and scripts

```sh
curl -H "Authorization: Bearer $NITRO_TOKEN" \
     https://repo.example.com/api/user/me
```

### Maven

Maven only speaks Basic auth, so the token goes in the password field. The username is not checked
when the password is a token, but something has to be there:

```xml
<server>
  <id>nitro-releases</id>
  <username>ci</username>
  <password>${env.NITRO_TOKEN}</password>
</server>
```

Requiring this is a per-repository setting — turn on `must_use_auth_token_for_push` and a real
password is refused for deploys.

### npm

`npm login` gets you a token automatically; you rarely need to make one by hand. If you do — for a
CI pipeline that cannot open a browser — put it straight in `.npmrc`:

```ini
//repo.example.com/repositories/local/npm/:_authToken=${NITRO_TOKEN}
```

See [npm → Authenticating](/repositories/npm/authenticating/).

## Expiry

`expires_in_days` sets a lifetime; omitting it makes a token that never expires. Give CI tokens an
expiry. The column has existed since the beginning but nothing wrote to it until recently, so
tokens created by older versions are all non-expiring.

## After a leak

**Profile → Tokens → Revoke all** deletes every token you own in one action, and reports how many
went. It needs a browser session, so the compromised token cannot survive by being the thing making
the request.

An administrator can do the same to another user from that user's page, which is what you want when
the person who leaked the token is not the person noticing.

Revoking is not reversible and it is not selective — every integration that user owns stops working
until new tokens are issued. That is the right trade when you do not know which token leaked.
