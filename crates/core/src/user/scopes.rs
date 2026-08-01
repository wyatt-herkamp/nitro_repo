use derive_more::derive::From;
use nr_macros::Scopes;
use serde::Serialize;
use sqlx::prelude::Type;
use strum::EnumIter;
use thiserror::Error;
use utoipa::ToSchema;
#[derive(Debug, Error, From)]
#[error("Invalid Scope: {0}")]
pub struct InvalidScope(pub String);

/// What an API token is allowed to do.
///
/// Only four of these existed — `ReadRepository`, `WriteRepository`, `EditRepository` and
/// `UpdatePassword` — so every other API route was reachable by any token belonging to a user with
/// the underlying permission. A token minted to let CI publish one artifact could also delete
/// repositories, create users and rewrite system settings.
///
/// A scope is a *ceiling*, never a grant: holding one still requires the user behind the token to
/// have the permission. That is why a scope for an administrative action is safe to offer to
/// everyone — a non-admin's token carrying `DeleteRepository` still cannot delete anything.
///
/// **The variant names are persisted** in `user_auth_token_scopes.scope` as text, so renaming one
/// silently invalidates every token that carries it. Adding is safe; renaming is not.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Type, ToSchema, EnumIter, Scopes)]
#[sqlx(type_name = "TEXT")]
pub enum NRScope {
    /// Can read all repositories the user has access to
    #[scope(title = "Read Repository", parent = "Repository")]
    ReadRepository,
    /// Can write to all repositories the user has access to
    #[scope(title = "Write Repository", parent = "Repository")]
    WriteRepository,
    /// Can edit the settings of repositories the user has access to
    #[scope(title = "Edit Repository", parent = "Repository")]
    EditRepository,
    /// Can create new repositories
    #[scope(title = "Create Repository", parent = "Repository", requires_system)]
    CreateRepository,
    /// Can delete repositories, and everything stored in them
    #[scope(title = "Delete Repository", parent = "Repository", requires_system)]
    DeleteRepository,

    /// Can list storages and read their settings
    #[scope(title = "Read Storage", parent = "Storage")]
    ReadStorage,
    /// Can create, edit and delete storages
    #[scope(title = "Manage Storage", parent = "Storage", requires_system)]
    ManageStorage,

    /// Update your password. Changing a password also requires a browser session, so a token
    /// alone is never sufficient
    #[scope(title = "Update Password", parent = "User")]
    UpdatePassword,
    /// Read your own profile, permissions and sessions
    #[scope(title = "Read Profile", parent = "User")]
    ReadSelf,

    /// Can list users and read their profiles
    #[scope(
        title = "Read Users",
        parent = "User Management",
        requires_user_manager
    )]
    ReadUser,
    /// Can create new users
    #[scope(
        title = "Create Users",
        parent = "User Management",
        requires_user_manager
    )]
    CreateUser,
    /// Can edit users, including their permissions
    #[scope(
        title = "Edit Users",
        parent = "User Management",
        requires_user_manager
    )]
    EditUser,
    /// Can delete users
    #[scope(
        title = "Delete Users",
        parent = "User Management",
        requires_user_manager
    )]
    DeleteUser,

    /// Can read instance-wide settings
    #[scope(title = "Read System Settings", parent = "System", requires_system)]
    ReadSystem,
    /// Can change instance-wide settings
    #[scope(title = "Edit System Settings", parent = "System", requires_admin)]
    EditSystem,
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash, ToSchema)]
pub struct ScopeDescription {
    pub key: NRScope,
    pub description: &'static str,
    pub name: &'static str,
    pub parent: Option<&'static str>,
    pub requires_user_manager: bool,
    pub requires_admin: bool,
    pub requires_system: bool,
}

impl Default for ScopeDescription {
    fn default() -> Self {
        Self {
            key: NRScope::ReadRepository,
            description: "",
            name: "",
            parent: None,
            requires_user_manager: false,
            requires_admin: false,
            requires_system: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    /// The variant name is what is written to `user_auth_token_scopes.scope`. Renaming one
    /// silently invalidates every existing token that carries it, so the four that shipped are
    /// pinned here.
    #[test]
    fn persisted_scope_names_do_not_change() {
        for (scope, expected) in [
            (NRScope::ReadRepository, "ReadRepository"),
            (NRScope::WriteRepository, "WriteRepository"),
            (NRScope::EditRepository, "EditRepository"),
            (NRScope::UpdatePassword, "UpdatePassword"),
        ] {
            assert_eq!(scope.as_ref(), expected);
            assert_eq!(NRScope::try_from(expected).unwrap(), scope);
        }
    }

    #[test]
    fn every_scope_round_trips_through_its_name() {
        for scope in NRScope::iter() {
            let name = scope.as_ref();
            assert_eq!(
                NRScope::try_from(name).unwrap_or_else(|err| panic!("{name} did not parse: {err}")),
                scope
            );
        }
    }

    #[test]
    fn every_scope_is_described() {
        for scope in NRScope::iter() {
            let description = scope.description();
            assert!(!description.name.is_empty(), "{scope:?} has no title");
            assert!(
                !description.description.trim().is_empty(),
                "{scope:?} has no doc comment, so the UI would show a blank explanation"
            );
            assert!(description.parent.is_some(), "{scope:?} has no group");
        }
    }
}
