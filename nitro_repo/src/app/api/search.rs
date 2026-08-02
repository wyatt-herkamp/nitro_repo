//! Artifact search.
//!
//! Two ways in, one query layer behind them. #506 asks for a "repository searching system" and
//! #411 asks for a query language; they are the same thing seen from two distances, so a simple
//! text search compiles to the same AST an explicit query does.
//!
//! Results are filtered by what the caller can read. A search that returned artifacts from a
//! private repository would be a way to enumerate it without ever fetching a file.
use axum::{
    extract::{Query as QueryParams, State},
    response::{IntoResponse, Response},
};
use nr_aql::{
    Field, ParseError,
    ast::{Query, Value},
    lexer::Operator,
    sql::{Binding, CompiledQuery},
};
use nr_core::{
    repository::Visibility,
    user::permissions::{HasPermissions, RepositoryActions},
};
use serde::{Deserialize, Serialize};
use sqlx::{AssertSqlSafe, Row};
use tracing::instrument;
use utoipa::{IntoParams, OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    app::{NitroRepo, authentication::Authentication},
    error::InternalError,
    utils::ResponseBuilder,
};

/// The most a single search will return. A query with no filters matches everything, and without
/// a ceiling that is a way to pull the entire index in one request.
const MAX_LIMIT: i64 = 200;
const DEFAULT_LIMIT: i64 = 50;

#[derive(OpenApi)]
#[openapi(
    paths(search, search_fields),
    components(schemas(SearchResponse, SearchResult, SearchFieldDescription))
)]
pub struct SearchAPI;

pub fn search_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/", axum::routing::get(search))
        .route("/fields", axum::routing::get(search_fields))
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchRequest {
    /// A query in the artifact query language, e.g. `scope == dev.kingtux and version ~= 1.*`.
    ///
    /// When absent, `text` is used instead.
    pub query: Option<String>,
    /// A plain search term, matched against project key, name and description.
    pub text: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    /// How many rows were returned. Not a total — counting every match costs a second scan, and
    /// nothing in the UI needs it.
    pub count: usize,
    /// The query that ran, after a plain `text` search was expanded into one.
    pub query: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResult {
    pub project_id: Uuid,
    pub version_id: Uuid,
    pub repository_id: Uuid,
    pub repository: String,
    pub storage: String,
    pub project_key: String,
    pub name: String,
    pub scope: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub release_type: String,
    pub version_path: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchFieldDescription {
    pub name: &'static str,
    /// Whether `>`/`<` may be used on it.
    pub orderable: bool,
}

/// The fields a query may use, so the UI can offer them rather than hardcoding a second list.
#[utoipa::path(
    get,
    path = "/fields",
    responses((status = 200, description = "Queryable fields", body = [SearchFieldDescription]))
)]
pub async fn search_fields() -> Response {
    let fields: Vec<SearchFieldDescription> = Field::all()
        .iter()
        .map(|field| SearchFieldDescription {
            name: field.as_str(),
            orderable: field.is_temporal(),
        })
        .collect();
    ResponseBuilder::ok().json(&fields)
}

/// Expands a plain search term into a query.
///
/// `foo` becomes `project ~= *foo* or name ~= *foo* or description ~= *foo*`, which is what
/// someone typing a word into a search box means.
fn text_query(text: &str) -> Query {
    let pattern = Value::Text(format!("*{text}*"));
    let matches = |field: Field| Query::Comparison {
        field,
        operator: Operator::Matches,
        value: pattern.clone(),
    };
    Query::or(
        matches(Field::Project),
        Query::or(matches(Field::Name), matches(Field::Description)),
    )
}

#[utoipa::path(
    get,
    path = "/",
    params(SearchRequest),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "The query could not be parsed"),
    )
)]
#[instrument(skip(site))]
pub async fn search(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    QueryParams(request): QueryParams<SearchRequest>,
) -> Result<Response, InternalError> {
    let (query, source) = match (&request.query, &request.text) {
        (Some(query), _) if !query.trim().is_empty() => match nr_aql::parse(query) {
            Ok(parsed) => (parsed, query.clone()),
            Err(error) => return Ok(parse_error_response(error)),
        },
        (_, Some(text)) if !text.trim().is_empty() => {
            (text_query(text.trim()), format!("text: {text}"))
        }
        // Neither given: everything, subject to the limit and to what the caller can read.
        _ => (
            Query::Comparison {
                field: Field::Project,
                operator: Operator::Matches,
                value: Value::Text("*".to_owned()),
            },
            "*".to_owned(),
        ),
    };

    let compiled = CompiledQuery::compile(&query);
    let limit = request.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = request.offset.unwrap_or(0).max(0);

    // The predicate's placeholders are `$1..$n`; the limit and offset take the two after them.
    let limit_placeholder = compiled.bindings.len() + 1;
    let offset_placeholder = compiled.bindings.len() + 2;
    let statement = format!(
        "SELECT project_versions.id AS version_id, \
                project_versions.version, \
                project_versions.release_type, \
                project_versions.path AS version_path, \
                project_versions.created_at, \
                project_versions.updated_at, \
                projects.id AS project_id, \
                projects.key AS project_key, \
                projects.name AS project_name, \
                projects.scope, \
                projects.description, \
                repositories.id AS repository_id, \
                repositories.name AS repository_name, \
                repositories.visibility, \
                storages.name AS storage_name \
         {from} WHERE {predicate} \
         ORDER BY project_versions.updated_at DESC \
         LIMIT ${limit_placeholder} OFFSET ${offset_placeholder}",
        from = CompiledQuery::FROM_CLAUSE,
        predicate = compiled.predicate,
    );

    // sqlx 0.9 only accepts `&'static str` without an assertion. This statement is assembled here
    // and every user-supplied value reaches it as a `$n` binding below — `compiled.predicate` is
    // built from `CompiledQuery`, which emits placeholders rather than interpolating input — so
    // there is nothing in the string that came from the request.
    let mut sql_query = sqlx::query(AssertSqlSafe(statement));
    for binding in &compiled.bindings {
        sql_query = match binding {
            Binding::Text(text) => sql_query.bind(text),
            Binding::Number(number) => sql_query.bind(number),
        };
    }
    // Over-fetch, because rows the caller cannot read are dropped afterwards and would otherwise
    // eat into the page they asked for. Bounded so a caller cannot make this arbitrarily large.
    let fetch_limit = (limit * 4).min(MAX_LIMIT * 4);
    let rows = sql_query
        .bind(fetch_limit)
        .bind(offset)
        .fetch_all(site.as_ref())
        .await?;

    let mut results = Vec::new();
    for row in rows {
        if results.len() as i64 >= limit {
            break;
        }
        let visibility: Visibility = row.try_get("visibility")?;
        let repository_id: Uuid = row.try_get("repository_id")?;
        // A private or hidden repository's contents are only searchable by someone who can read
        // it — otherwise search is a way to enumerate a repository without fetching from it.
        if !matches!(visibility, Visibility::Public)
            && !auth
                .has_action(RepositoryActions::Read, repository_id, site.as_ref())
                .await?
        {
            continue;
        }
        results.push(SearchResult {
            project_id: row.try_get("project_id")?,
            version_id: row.try_get("version_id")?,
            repository_id,
            repository: row.try_get("repository_name")?,
            storage: row.try_get("storage_name")?,
            project_key: row.try_get("project_key")?,
            name: row.try_get("project_name")?,
            scope: row.try_get("scope")?,
            description: row.try_get("description")?,
            version: row.try_get("version")?,
            release_type: row.try_get("release_type")?,
            version_path: row.try_get("version_path")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        });
    }

    Ok(ResponseBuilder::ok().json(&SearchResponse {
        count: results.len(),
        results,
        query: source,
    }))
}

/// Reports a parse failure with the position it happened at, so the UI can underline it.
fn parse_error_response(error: ParseError) -> Response {
    #[derive(Serialize)]
    struct ParseErrorResponse {
        error: String,
        start: Option<usize>,
        end: Option<usize>,
    }
    let span = error.span();
    ResponseBuilder::bad_request()
        .json(&ParseErrorResponse {
            error: error.to_string(),
            start: span.map(|span| span.start),
            end: span.map(|span| span.end),
        })
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A word typed into a search box should look in the places a person means.
    #[test]
    fn plain_text_expands_to_a_query_over_the_obvious_fields() {
        let compiled = CompiledQuery::compile(&text_query("tms"));
        assert_eq!(compiled.bindings.len(), 3);
        for binding in &compiled.bindings {
            assert_eq!(binding, &Binding::Text("%tms%".to_owned()));
        }
        assert!(compiled.predicate.contains("projects.key"));
        assert!(compiled.predicate.contains("projects.name"));
        assert!(compiled.predicate.contains("projects.description"));
    }

    /// The statement is built by format!, so the placeholder numbering has to follow the
    /// bindings — an off-by-one here silently pages by the wrong value.
    #[test]
    fn limit_and_offset_follow_the_query_bindings() {
        let compiled = CompiledQuery::compile(&text_query("tms"));
        assert_eq!(compiled.bindings.len(), 3);
        assert_eq!(compiled.bindings.len() + 1, 4);
        assert_eq!(compiled.bindings.len() + 2, 5);
    }
}
