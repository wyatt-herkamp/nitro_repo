use std::fmt::{Debug, Display};

use chrono::NaiveDate;
use derive_more::derive::From;
use serde::{Deserialize, Serialize};
use sqlx::query_builder::Separated;
use tracing::{instrument, trace, warn};
use utoipa::ToSchema;
mod string_param;
pub use string_param::*;

use super::{ColumnType, SQLOrder};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
#[schema(examples(
    date_param_examples::between_dates,
    date_param_examples::on,
    date_param_examples::list_in
))]
pub enum DateParamType {
    /// A list of dates in a month or week
    ListIn {
        key: ListIn,
        #[serde(alias = "week", alias = "month")]
        week_or_month: i32,
        /// The year to filter by
        ///
        /// Currently not implemented
        year: Option<i32>,
    },
    /// A date between two dates
    BetweenDates {
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    },
    /// A date on a specific date
    On(NaiveDate),
}
mod date_param_examples {
    use chrono::NaiveDate;

    use super::{DateParamType, ListIn};
    pub fn between_dates() -> DateParamType {
        DateParamType::BetweenDates {
            start: Some(NaiveDate::from_ymd_opt(2021, 1, 1).expect("Invalid Date")),
            end: Some(NaiveDate::from_ymd_opt(2021, 1, 31).expect("Invalid Date")),
        }
    }
    pub fn on() -> DateParamType {
        DateParamType::On(NaiveDate::from_ymd_opt(2021, 1, 1).expect("Invalid Date"))
    }
    pub fn list_in() -> DateParamType {
        DateParamType::ListIn {
            key: ListIn::Month,
            week_or_month: 1,
            year: None,
        }
    }
}

impl From<NaiveDate> for DateParamType {
    fn from(date: NaiveDate) -> Self {
        Self::On(date)
    }
}
impl From<(Option<NaiveDate>, Option<NaiveDate>)> for DateParamType {
    fn from((start, end): (Option<NaiveDate>, Option<NaiveDate>)) -> Self {
        Self::BetweenDates { start, end }
    }
}
impl DateParamType {
    #[instrument(name = "DateParamType::push_to_seperated", skip(separated))]
    pub fn push_to_seperated<C: Display + Debug>(
        &self,
        column_name: C,
        separated: &mut Separated<'_, '_, sqlx::Postgres, &str>,
    ) {
        match self {
            DateParamType::ListIn {
                key,
                week_or_month,
                year,
            } => {
                // TODO: Add year to the query
                warn!(?year, "Year is not implemented yet");
                match key {
                    ListIn::Month => {
                        separated.push(format!("EXTRACT(MONTH FROM {}) = ", column_name));
                        separated.push_bind_unseparated(*week_or_month);
                    }
                    ListIn::Week => {
                        separated.push(format!("EXTRACT(WEEK FROM {}) = ", column_name));
                        separated.push_bind_unseparated(*week_or_month);
                    }
                }
            }
            DateParamType::BetweenDates { start, end } => match (start, end) {
                (Some(start), Some(end)) => {
                    separated.push(format!("{} BETWEEN ", column_name));
                    separated.push_bind_unseparated(*start);
                    separated.push_bind(*end);
                }
                (Some(start), None) => {
                    separated.push(format!("{} >= ", column_name));
                    separated.push_bind_unseparated(*start);
                }
                (None, Some(end)) => {
                    separated.push(format!("{} <= ", column_name));
                    separated.push_bind_unseparated(*end);
                }
                _ => {
                    trace!(?column_name, "No dates provided for between");
                }
            },
            DateParamType::On(date) => {
                separated.push(format!("{} = ", column_name));
                separated.push_bind_unseparated(*date);
            }
        }
    }
}
/// The list in type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, ToSchema)]
pub enum ListIn {
    /// The month
    Month,
    /// The week
    Week,
}
pub trait QueryOrderingColumn {
    type ColumnType: ColumnType;
    /// The column to order by
    fn column(&self) -> Self::ColumnType;
    /// The name of the column
    ///
    /// This just maps to the column name of the column
    #[inline]
    fn column_name(&self) -> &'static str {
        self.column().column_name()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "ordering", content = "column")]
pub enum QueryOrdering<C: QueryOrderingColumn> {
    /// Order with the column in ascending order
    Ascending(C),
    /// Order with the column in descending order
    Descending(C),
}
impl<C: QueryOrderingColumn> QueryOrdering<C> {
    /// Gets the [SQLOrder] from the ordering
    pub fn sql_order(&self) -> SQLOrder {
        match self {
            QueryOrdering::Ascending(_) => SQLOrder::Ascending,
            QueryOrdering::Descending(_) => SQLOrder::Descending,
        }
    }
    /// Gets the column from the ordering
    pub fn column(&self) -> &C {
        match self {
            QueryOrdering::Ascending(column) => column,
            QueryOrdering::Descending(column) => column,
        }
    }

    pub fn sql_with_order_by(&self) -> QueryOrderingWithOrderByStatement<C> {
        QueryOrderingWithOrderByStatement(self)
    }
}
impl<C> Default for QueryOrdering<C>
where
    C: QueryOrderingColumn + Default,
{
    fn default() -> Self {
        Self::Descending(C::default())
    }
}
impl<C: QueryOrderingColumn> Display for QueryOrdering<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryOrdering::Ascending(column) => write!(f, "{} ASC", column.column_name()),
            QueryOrdering::Descending(column) => {
                write!(f, "{} DESC", column.column_name())
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, From)]
pub struct QueryOrderingWithOrderByStatement<'ordering, C: QueryOrderingColumn>(
    pub &'ordering QueryOrdering<C>,
);
impl<C: QueryOrderingColumn> Display for QueryOrderingWithOrderByStatement<'_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ORDER BY {}", self.0)
    }
}
#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use sqlx::Postgres;

    use super::QueryOrderingColumn;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TestDateColumn;
    impl QueryOrderingColumn for TestDateColumn {
        type ColumnType = TestDateColumn;
        fn column(&self) -> Self::ColumnType {
            *self
        }
    }
    impl super::ColumnType for TestDateColumn {
        fn column_name(&self) -> &'static str {
            "date"
        }

        fn all() -> Vec<Self>
        where
            Self: Sized,
        {
            vec![Self]
        }

        fn all_static() -> &'static [Self]
        where
            Self: Sized,
        {
            &[Self]
        }
    }
    impl Display for TestDateColumn {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "date")
        }
    }
    #[test]
    pub fn test_date_between() {
        let mut query =
            sqlx::query_builder::QueryBuilder::<Postgres>::new("SELECT * FROM table WHERE ");

        let mut separated = query.separated(" AND ");

        {
            let date = super::DateParamType::BetweenDates {
                start: Some(chrono::NaiveDate::from_ymd_opt(2021, 1, 1).expect("Invalid Date")),
                end: Some(chrono::NaiveDate::from_ymd_opt(2021, 1, 31).expect("Invalid Date")),
            };
            date.push_to_seperated(TestDateColumn, &mut separated);
        }
        {
            let date = super::DateParamType::BetweenDates {
                start: Some(chrono::NaiveDate::from_ymd_opt(2021, 1, 1).expect("Invalid Date")),
                end: None,
            };
            date.push_to_seperated(TestDateColumn, &mut separated);
        }
        {
            let date = super::DateParamType::BetweenDates {
                start: None,
                end: Some(chrono::NaiveDate::from_ymd_opt(2021, 1, 31).expect("Invalid Date")),
            };
            date.push_to_seperated(TestDateColumn, &mut separated);
        }

        println!("{}", query.sql());
    }

    #[test]
    pub fn test_date_on() {
        let mut query =
            sqlx::query_builder::QueryBuilder::<Postgres>::new("SELECT * FROM table WHERE ");

        let mut separated = query.separated(" AND ");

        {
            let date = super::DateParamType::On(
                chrono::NaiveDate::from_ymd_opt(2021, 1, 1).expect("Invalid Date"),
            );
            date.push_to_seperated(TestDateColumn, &mut separated);
        }

        println!("{}", query.sql());
    }

    #[test]
    pub fn test_date_list_in() {
        let mut query =
            sqlx::query_builder::QueryBuilder::<Postgres>::new("SELECT * FROM table WHERE ");

        let mut separated = query.separated(" AND ");

        {
            let date = super::DateParamType::ListIn {
                key: super::ListIn::Month,
                week_or_month: 1,
                year: None,
            };
            date.push_to_seperated(TestDateColumn, &mut separated);
        }
        {
            let date = super::DateParamType::ListIn {
                key: super::ListIn::Week,
                week_or_month: 1,
                year: None,
            };
            date.push_to_seperated(TestDateColumn, &mut separated);
        }

        println!("{}", query.sql());
    }
}
