//! Pagination Related Types and Functions

use std::{fmt::Display, ops::Deref};

use derive_more::derive::{From, Into};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use tracing::{debug, error, instrument, trace};
use utoipa::{IntoParams, ToSchema};

use crate::database::DBResult;
/// A SQL Tool that supports pagination
pub trait PaginationSupportingTool {
    /// Set the limit for the query
    fn limit(&mut self, limit: i32) -> &mut Self;
    /// Set the offset for the query
    fn offset(&mut self, offset: i32) -> &mut Self;
    /// Set the page parameters for the query
    fn page_params(&mut self, page_params: impl Into<PageParams>) -> &mut Self {
        let page_params = page_params.into();
        self.limit(page_params.limit()).offset(page_params.offset())
    }
}
/// Parameters for pagination
///
/// Includes the page size and the page number
///
/// # Note
/// Passing a page number less than 1 or equal to I32::MAX might result in all items being returned
/// This is dependent on the request handler
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, From, Into, Serialize, Deserialize, ToSchema, IntoParams,
)]
#[serde(default)]
#[into_params(parameter_in = Query)]
pub struct PageParams {
    /// The number of items per page
    #[param(default = 10)]
    #[serde(alias = "limit")]
    pub page_size: i32,
    /// The page number
    #[param(default = 1)]
    #[serde(alias = "page")]
    pub page_number: i32,
}
impl PageParams {
    /// If the page size is greater than the max argument it is set to the max argument
    pub fn max_page_size(&mut self, max: i32) {
        self.page_size = self.page_size.min(max);
    }
    /// Convert to a SQL display
    #[inline]
    pub fn sql(&self) -> PageParamsSQLDisplay {
        self.into()
    }
    /// Calculate the number of pages based on the total number of items
    #[inline]
    pub fn number_of_pages(&self, total: i64) -> i32 {
        (total as f64 / self.page_size as f64).ceil() as i32
    }
    #[inline]
    pub fn limit(&self) -> i32 {
        self.page_size
    }
    /// Requests start at 1.
    /// However, offset starts at 0.
    ///
    /// This function returns the index of the page.
    #[inline]
    pub fn page_index(&self) -> i32 {
        (self.page_number - 1).max(0)
    }
    /// Requests start at 1.
    #[inline]
    pub fn offset(&self) -> i32 {
        self.page_size * self.page_index()
    }
    /// Create a paginated response
    pub fn create_result<T>(&self, total: i64, data: Vec<T>) -> PaginatedResponse<T> {
        PaginatedResponse {
            total_pages: self.number_of_pages(total),
            total,
            data,
        }
    }
}
impl Default for PageParams {
    fn default() -> Self {
        Self {
            page_size: 10,
            page_number: 1,
        }
    }
}

/// A display for the page parameters in SQL
///
/// Writes the limit and offset
///
/// If offset is 0, it is not included
///
/// If limit is i32::MAX or 0 or less, it is not included
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageParamsSQLDisplay<'params>(&'params PageParams);
impl Display for PageParamsSQLDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let limit = self.0.limit();
        if limit > 0 && limit != i32::MAX {
            write!(f, " LIMIT {}", limit)?;
        }
        let offset = self.0.offset();
        if offset > 0 {
            write!(f, " OFFSET {}", offset)?;
        }
        Ok(())
    }
}
impl<'params> From<&'params PageParams> for PageParamsSQLDisplay<'params> {
    #[inline]
    fn from(value: &'params PageParams) -> Self {
        Self(value)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, From)]
pub enum QueryLimit {
    PageAndSize(PageParams),
    NoLimit,
}

/// A paginated response
///
/// Includes a total number of pages and the total number of items
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// The number of items per page
    pub total_pages: i32,
    /// The total number of items in the query
    pub total: i64,
    /// The data for the current page
    pub data: Vec<T>,
}
impl<T> PaginatedResponse<T>
where
    T: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync,
{
    #[instrument(skip(rows, full_count_column))]
    fn from_rows(rows: Vec<PgRow>, full_count_column: &str) -> DBResult<Option<(Vec<T>, i64)>> {
        if rows.is_empty() {
            debug!("No rows found");
            return Ok(None);
        }
        let total = match rows[0].try_get::<i64, _>(full_count_column) {
            Ok(ok) => {
                trace!(?ok, ?full_count_column, "Total Count");
                ok
            }
            Err(err) => {
                error!(?err, ?full_count_column, "Failed to get total count");
                0
            }
        };
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(T::from_row(&row)?);
        }
        Ok(Some((result, total)))
    }
    #[instrument(skip(rows, page_params, full_count_column))]
    pub fn from_rows_with_page_params(
        rows: Vec<PgRow>,
        page_params: &PageParams,
        full_count_column: &str,
    ) -> DBResult<Self> {
        let (result, total) = match Self::from_rows(rows, full_count_column)? {
            Some(ok) => ok,
            None => return Ok(Self::default()),
        };
        let total_pages = page_params.number_of_pages(total);
        Ok(Self {
            total,
            total_pages,
            data: result,
        })
    }
    #[instrument(skip(rows, full_count_column))]
    pub fn from_rows_no_page(rows: Vec<PgRow>, full_count_column: &str) -> DBResult<Self> {
        let (result, total) = match Self::from_rows(rows, full_count_column)? {
            Some(ok) => ok,
            None => return Ok(Self::default()),
        };
        Ok(Self {
            total,
            total_pages: 1,
            data: result,
        })
    }
    pub fn from_rows_with_query_limit(
        rows: Vec<PgRow>,
        query_limit: &QueryLimit,
        full_count_column: &str,
    ) -> DBResult<Self> {
        match &query_limit {
            QueryLimit::PageAndSize(page_params) => {
                Self::from_rows_with_page_params(rows, page_params, full_count_column)
            }
            QueryLimit::NoLimit => Self::from_rows_no_page(rows, full_count_column),
        }
    }
}
impl<T> Deref for PaginatedResponse<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T> Default for PaginatedResponse<T> {
    fn default() -> Self {
        Self {
            total_pages: 0,
            total: 0,
            data: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageParams;

    #[test]
    pub fn page_params() {
        {
            let params = PageParams {
                page_size: 10,
                page_number: 10,
            };
            assert_eq!(params.page_index(), 9);
            assert_eq!(params.offset(), 90);
        }
    }
}
