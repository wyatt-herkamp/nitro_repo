use uuid::Uuid;

use super::{DBProject, DBProjectColumn};
use crate::database::prelude::*;

/// A partial update to a project row.
///
/// This file was empty, so there was no way to change a project after creating it. That is why
/// `npm/utils.rs` carries a `// TODO: Update` and returns an existing project untouched: a
/// re-publish carrying a new description or name silently kept the old one forever.
///
/// `None` means "leave alone". The nested `Option` on the nullable columns separates that from
/// "set to null" — without it there would be no way to clear a description once set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateProject {
    pub scope: Option<Option<String>>,
    pub name: Option<Option<String>>,
    pub description: Option<Option<String>>,
    /// The storage path the project lives at.
    pub path: Option<String>,
}

impl UpdateProject {
    /// Whether this would change anything, so a caller can skip a pointless round trip.
    pub fn is_empty(&self) -> bool {
        self.scope.is_none()
            && self.name.is_none()
            && self.description.is_none()
            && self.path.is_none()
    }

    pub async fn update(&self, project_id: Uuid, database: &PgPool) -> DBResult<()> {
        if self.is_empty() {
            return Ok(());
        }
        let mut update = self.build(project_id);
        update.query().execute(database).await?;
        Ok(())
    }

    fn build<'args>(&'args self, project_id: Uuid) -> UpdateQueryBuilder<'args> {
        let mut update = UpdateQueryBuilder::new(DBProject::table_name());
        update
            .set(DBProjectColumn::UpdatedAt, SqlFunctionBuilder::now())
            // Scoped to one row. The version updater next door shipped without this and rewrote
            // every row in its table.
            .filter(DBProjectColumn::Id.equals(project_id.value()));

        if let Some(scope) = &self.scope {
            update.set(DBProjectColumn::Scope, scope.value());
        }
        if let Some(name) = &self.name {
            update.set(DBProjectColumn::Name, name.value());
        }
        if let Some(description) = &self.description {
            update.set(DBProjectColumn::Description, description.value());
        }
        if let Some(path) = &self.path {
            update.set(DBProjectColumn::Path, path.value());
        }
        update
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_is_scoped_to_one_row() {
        let update = UpdateProject {
            name: Some(Some("New Name".to_owned())),
            ..Default::default()
        };
        let id = Uuid::new_v4();
        let mut built = update.build(id);
        let sql = built.format_sql_query().to_owned();

        let (set_clause, where_clause) = sql
            .split_once(" WHERE ")
            .unwrap_or_else(|| panic!("no WHERE clause — this rewrites every row: {sql}"));
        assert!(
            !set_clause.contains("id ="),
            "statement assigns the primary key instead of filtering on it: {sql}"
        );
        assert!(where_clause.contains("id ="), "not filtered by id: {sql}");
    }

    /// An update with nothing set must not issue a statement at all.
    #[test]
    fn empty_update_is_a_no_op() {
        assert!(UpdateProject::default().is_empty());
        assert!(
            !UpdateProject {
                description: Some(None),
                ..Default::default()
            }
            .is_empty(),
            "clearing a field is a real change, not an empty update"
        );
    }
}
