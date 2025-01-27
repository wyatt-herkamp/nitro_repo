use std::fmt::{Debug, Display};
mod pagination;
pub mod query;
mod select;
mod traits;
mod where_sql;
pub use pagination::*;
pub use select::*;
use serde::{Deserialize, Serialize};
use sqlx::{query_builder::Separated, Encode, Postgres, Type};
use strum::{AsRefStr, Display};
pub use traits::*;
use utoipa::ToSchema;
pub use where_sql::*;
mod insert;
pub use insert::*;
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum NewOrId<T> {
    New(T),
    Id(i32),
}

pub trait SeperatedExt<'args> {
    fn push_and_bind<T>(&mut self, key: impl ColumnType, value: T)
    where
        T: Encode<'args, Postgres> + 'args + Type<Postgres>;
}
impl<'args> SeperatedExt<'args> for Separated<'_, 'args, sqlx::Postgres, &str> {
    fn push_and_bind<T>(&mut self, name: impl ColumnType, value: T)
    where
        T: Encode<'args, Postgres> + 'args + Type<Postgres>,
    {
        self.push(format!("{} = ", name.column_name()));
        self.push_bind_unseparated(value);
    }
}
pub struct FunctionCallColumn<C> {
    pub function_name: &'static str,
    pub column: C,
}

pub trait TableType {
    type Columns: ColumnType;
    fn table_name() -> &'static str
    where
        Self: Sized;
}
// Implement TableQuery for any type that implements TableType

impl<T> TableQuery for T
where
    T: TableType,
{
    type Table = T;

    fn columns() -> Vec<<Self::Table as TableType>::Columns> {
        <Self::Table as TableType>::Columns::all()
    }
}
pub trait TableQuery {
    type Table: TableType;

    fn columns() -> Vec<<Self::Table as TableType>::Columns>
    where
        Self: Sized;
}
/// A workaround for https://github.com/rust-lang/rust/issues/20041 being unstable
pub mod rust_unstable_workaround {
    use super::{ColumnType, TableQuery, TableType};

    pub trait HasColumns {
        type Columns: ColumnType;
        fn columns() -> Vec<Self::Columns>
        where
            Self: Sized;
    }
    pub trait HasTableName {
        fn table_name() -> &'static str
        where
            Self: Sized;
    }

    impl<T: TableQuery> HasTableName for T {
        fn table_name() -> &'static str {
            <T::Table as TableType>::table_name()
        }
    }

    impl<T: TableQuery> HasColumns for T {
        type Columns = <T::Table as TableType>::Columns;

        fn columns() -> Vec<Self::Columns> {
            T::columns()
        }
    }
}
pub trait ColumnType: Debug {
    fn column_name(&self) -> &'static str;

    fn format_column_with_prefix(&self, prefix: &str) -> String {
        format!("{}.{}", prefix, self.column_name())
    }
    fn all() -> Vec<Self>
    where
        Self: Sized;

    fn all_static() -> &'static [Self]
    where
        Self: Sized;

    fn lower(&self) -> FunctionCallColumn<Self>
    where
        Self: Sized + Copy,
    {
        FunctionCallColumn {
            function_name: "LOWER",
            column: *self,
        }
    }
    fn upper(&self) -> FunctionCallColumn<Self>
    where
        Self: Sized + Copy,
    {
        FunctionCallColumn {
            function_name: "UPPER",
            column: *self,
        }
    }
}
pub fn concat_columns<T>(columns: &[T], prefix: Option<&str>) -> String
where
    T: ColumnType,
{
    if let Some(prefix) = prefix {
        columns
            .iter()
            .map(|column| column.format_column_with_prefix(prefix))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        columns
            .iter()
            .map(|column| column.column_name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SQLComparison {
    /// Equals
    ///
    /// `=`
    Equals,
    /// [LIKE](https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE)
    ///
    /// `LIKE`
    Like,
    /// Not Equals
    ///
    /// `!=`
    NotEquals,
    // In,
    ArrayIn,
}
impl Display for SQLComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equals => write!(f, "="),
            Self::NotEquals => write!(f, "!="),
            Self::Like => write!(f, "LIKE"),
            Self::ArrayIn => write!(f, "IN"),
        }
    }
}
/// SQL Ordering
#[derive(Debug, Clone, Copy, PartialEq, Display, AsRefStr)]
pub enum SQLOrder {
    #[strum(serialize = "ASC")]
    Ascending,
    #[strum(serialize = "DESC")]
    Descending,
}
/// SQL And Or
#[derive(Debug, Clone, Copy, PartialEq, Display, AsRefStr)]
pub enum AndOr {
    #[strum(serialize = "AND")]
    And,
    #[strum(serialize = "OR")]
    Or,
}
