use std::future::Future;

use sqlx::{
    Arguments, Database, Decode, FromRow, Postgres, Type,
    postgres::PgRow,
    query::{Query, QueryAs, QueryScalar},
};
use tracing::trace;

use crate::database::DBResult;

/// A sql tool that has [Arguments](sqlx::Arguments) that can be used to build a query.
///
/// Arguments being the values that will be used to fill in the placeholders in the query.
///
/// # Note
/// The tools are highly inspired by the QueryBuilder in sqlx.
///
/// This means it stores arguments in an [Option] and can be removed from the type and may cause panics. if removed and then the item is used again
pub trait HasArguments<'args> {
    /// Takes the arguments list or panics if it is not available.
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args>;
    /// Borrows the arguments list or panics if it is not available.
    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args>;
    /// Pushes an argument to the arguments list.
    fn push_argument<T>(&mut self, value: T) -> usize
    where
        T: 'args + sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres>,
    {
        let arguments = self.borrow_arguments_or_error();
        arguments.add(value).expect("Failed to add argument");
        arguments.len()
    }
}
/// A base Query TOol type that can be used to build queries.
pub trait QueryTool<'args>: HasArguments<'args> {
    /// The SQL query that will be executed
    ///
    /// # Note
    /// This uses &mut self because some sql queries are not generated until the query is built but then its cached.
    fn sql(&mut self) -> &str;

    /// Builds a query that can be executed.
    ///
    /// See [sqlx::query_with] for more information.
    fn query(&mut self) -> Query<'_, Postgres, <Postgres as Database>::Arguments<'args>> {
        let args = self.take_arguments_or_error();
        let sql = self.sql();
        trace!(?sql, "Generated SQL");

        sqlx::query_with(sql, args)
    }
    /// Builds a query that can be executed and returns the results as a type.
    ///
    /// See [sqlx::query_as_with] for more information.
    fn query_as<T>(&mut self) -> QueryAs<'_, Postgres, T, <Postgres as Database>::Arguments<'args>>
    where
        T: for<'r> FromRow<'r, PgRow>,
    {
        let args = self.take_arguments_or_error();

        let sql = self.sql();
        trace!(?sql, "Generated SQL");
        sqlx::query_as_with(sql, args)
    }
    /// Builds a query that can be executed and returns the results as a scalar.
    ///
    /// See [sqlx::query_scalar_with] for more information.
    fn query_scalar<O>(
        &mut self,
    ) -> QueryScalar<'_, Postgres, O, <Postgres as Database>::Arguments<'args>>
    where
        (O,): for<'r> FromRow<'r, PgRow>,
    {
        let args = self.take_arguments_or_error();

        let sql = self.sql();
        trace!(?sql, "Generated SQL");
        sqlx::query_scalar_with(sql, args)
    }
}
/// Tools such as [SelectExists] and [SelectCount] that can be used to build queries that return a single value.
pub trait QueryScalarTool<'args>: QueryTool<'args> + Send {
    type Output: for<'r> Decode<'r, Postgres> + Type<Postgres> + Send + Unpin;
    /// Executes the query and returns the number of rows affected.
    ///
    /// See [sqlx::query] for more information.
    fn execute<'c, E>(&mut self, conn: E) -> impl Future<Output = DBResult<Self::Output>> + Send
    where
        E: sqlx::Executor<'c, Database = Postgres> + Send,
    {
        async move {
            let query = self.query_scalar();
            let result = query.fetch_one(conn).await?;
            Ok(result)
        }
    }
}
