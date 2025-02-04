use std::fmt::{Debug, Display};

use sqlx::{
    Arguments, Database, Encode, FromRow, Postgres, Type,
    query::{Query, QueryAs},
};
use tracing::trace;

use crate::database::prelude::*;
#[derive(Debug, Clone)]
enum Returning<C: ColumnType> {
    None,
    All,
    Columns(Vec<C>),
}
impl<C: ColumnType> Display for Returning<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::All => write!(f, " RETURNING *"),
            Self::Columns(columns) => {
                let columns = super::concat_columns(columns, None);
                write!(f, " RETURNING {}", columns)
            }
        }
    }
}
impl<C: ColumnType> Default for Returning<C> {
    fn default() -> Self {
        Self::None
    }
}
pub struct SimpleInsertQueryBuilder<'table, 'args, C: ColumnType> {
    columns_to_insert: Vec<C>,
    sql: Option<String>,
    returning: Returning<C>,
    table: &'table str,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'args, C> HasArguments<'args> for SimpleInsertQueryBuilder<'_, 'args, C>
where
    C: ColumnType,
{
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args> {
        self.arguments.take().expect("Arguments already taken")
    }

    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}

impl<C> Debug for SimpleInsertQueryBuilder<'_, '_, C>
where
    C: ColumnType + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleInsertQueryBuilder")
            .field("columns_to_insert", &self.columns_to_insert)
            .field("sql", &self.sql)
            .field("returning", &self.returning)
            .field("table", &self.table)
            .finish()
    }
}
impl<'table, 'args, C: ColumnType> SimpleInsertQueryBuilder<'table, 'args, C> {
    pub fn new(table: &'table str) -> Self {
        Self {
            table,
            arguments: Some(Default::default()),
            columns_to_insert: Vec::new(),
            sql: None,
            returning: Default::default(),
        }
    }
    fn push_value<T>(&mut self, value: T)
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        assert!(
            self.arguments.is_some(),
            "You called a query method already"
        );
        let arguments = self
            .arguments
            .as_mut()
            .expect("BUG: Arguments taken already");
        arguments.add(value).expect("Failed to add argument");
    }
    /// Insert a value into the query
    pub fn insert<T>(&mut self, column: C, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.sql = None;
        self.columns_to_insert.push(column);
        self.push_value(value);
        self
    }
    /// Will check if option is Some and insert the value if it is
    ///
    /// This will allow for the database to just use its default value if the option is None
    ///
    /// If you want to insert a NULL value use `insert` with `None`
    pub fn insert_option<T>(&mut self, column: C, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if let Some(value) = value {
            self.insert(column, value)
        } else {
            self
        }
    }

    pub fn return_all(&mut self) -> &mut Self {
        self.returning = Returning::All;
        self
    }
    pub fn return_columns(&mut self, columns: Vec<C>) -> &mut Self {
        self.returning = Returning::Columns(columns);
        self
    }
    fn gen_sql(&mut self) {
        let columns = super::concat_columns(&self.columns_to_insert, None);
        let placeholders = generate_placeholder_string(self.columns_to_insert.len());
        let sql = format!(
            "INSERT INTO {table} ({columns}) VALUES ({placeholders}){returning};",
            table = self.table,
            returning = self.returning,
        );

        self.sql = Some(sql);
    }
    pub fn query(&mut self) -> Query<'_, Postgres, <Postgres as Database>::Arguments<'args>> {
        let args = self.arguments.take().expect("BUG: Arguments taken already");
        let sql = self.sql();
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(?sql, "Generated SQL");
        }
        sqlx::query_with(sql, args)
    }
    pub fn query_as<T>(
        &mut self,
    ) -> QueryAs<'_, Postgres, T, <Postgres as Database>::Arguments<'args>>
    where
        T: for<'r> FromRow<'r, <Postgres as Database>::Row>,
    {
        let args = self.arguments.take().expect("BUG: Arguments taken already");

        let sql = self.sql();
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(?sql, "Generated SQL");
        }
        sqlx::query_as_with(sql, args)
    }
}
impl<'args, C> QueryTool<'args> for SimpleInsertQueryBuilder<'_, 'args, C>
where
    C: ColumnType,
{
    /// Generates the SQL for the query if not already generated
    ///
    /// Why do we need to store this in an Option?
    /// Because lifetime issues with the borrow checker
    fn sql(&mut self) -> &str {
        if self.sql.is_none() {
            self.gen_sql();
        }
        self.sql.as_ref().expect("BUG: SQL not generated")
    }
}
pub fn generate_placeholder_string(len: usize) -> String {
    (0..len)
        .map(|index| format!("${}", index + 1))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {}
