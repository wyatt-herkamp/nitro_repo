//! `GET /-/v1/search`, which backs `npm search`.
//!
//! `GetPath::Search` was declared and never constructed, so a search request fell through to the
//! tarball branch and came back as a 404 complaining about a missing package.
use nr_core::database::entities::project::{DBProject, versions::DBProjectVersion};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::{NPMRegistryError, utils::npm_time};
use crate::{
    repository::{RepoResponse, Repository},
    utils::ResponseBuilder,
};

/// npm's default page size, and the cap on what a caller can ask for. Without an upper bound a
/// single request could ask the database for every project in the registry.
const DEFAULT_SIZE: i64 = 20;
const MAX_SIZE: i64 = 250;

#[derive(Debug, Default, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub text: String,
    pub size: Option<i64>,
    pub from: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub objects: Vec<SearchObject>,
    pub total: i64,
    pub time: String,
}

#[derive(Debug, Serialize)]
pub struct SearchObject {
    pub package: SearchPackage,
    pub score: SearchScore,
    #[serde(rename = "searchScore")]
    pub search_score: f64,
}

#[derive(Debug, Serialize)]
pub struct SearchPackage {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub date: String,
    pub links: SearchLinks,
}

#[derive(Debug, Serialize)]
pub struct SearchLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
}

/// npm renders these as quality bars. Nothing here computes real quality signals, so they are
/// reported as a flat value rather than invented per package — a fabricated ranking would be worse
/// than an obviously uniform one.
#[derive(Debug, Serialize)]
pub struct SearchScore {
    #[serde(rename = "final")]
    pub final_score: f64,
    pub detail: SearchScoreDetail,
}

#[derive(Debug, Serialize)]
pub struct SearchScoreDetail {
    pub quality: f64,
    pub popularity: f64,
    pub maintenance: f64,
}

impl Default for SearchScore {
    fn default() -> Self {
        Self {
            final_score: 1.0,
            detail: SearchScoreDetail {
                quality: 1.0,
                popularity: 0.0,
                maintenance: 1.0,
            },
        }
    }
}

#[instrument(skip(repository))]
pub async fn handle_search(
    repository: &impl Repository,
    query: &str,
) -> Result<RepoResponse, NPMRegistryError> {
    let query: SearchQuery = serde_urlencoded::from_str(query).unwrap_or_default();
    let size = query.size.unwrap_or(DEFAULT_SIZE).clamp(1, MAX_SIZE);
    let from = query.from.unwrap_or(0).max(0);

    let site = repository.site();
    // npm sends the raw search text; `%` and `_` in it would otherwise act as LIKE wildcards.
    //
    // The `COLLATE "C"` below is not cosmetic: `projects.key` is declared with the nondeterministic
    // `ignoreCase` collation, and Postgres refuses `ILIKE` against one ("nondeterministic
    // collations are not supported for ILIKE"). Without it every search answered 500.
    let escaped = query
        .text
        .replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_");
    let pattern = format!("%{escaped}%");

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM projects
           WHERE repository_id = $1
             AND ((key COLLATE "C") ILIKE $2 OR COALESCE(description, '') ILIKE $2)"#,
    )
    .bind(repository.id())
    .bind(&pattern)
    .fetch_one(site.as_ref())
    .await?;

    let projects: Vec<DBProject> = sqlx::query_as(
        r#"SELECT * FROM projects
           WHERE repository_id = $1
             AND ((key COLLATE "C") ILIKE $2 OR COALESCE(description, '') ILIKE $2)
           ORDER BY updated_at DESC
           LIMIT $3 OFFSET $4"#,
    )
    .bind(repository.id())
    .bind(&pattern)
    .bind(size)
    .bind(from)
    .fetch_all(site.as_ref())
    .await?;

    let mut objects = Vec::with_capacity(projects.len());
    for project in projects {
        let versions = DBProjectVersion::get_all_versions(project.id, site.as_ref()).await?;
        let Some(newest) = versions.first() else {
            // A project row with no versions has nothing installable behind it.
            continue;
        };
        objects.push(SearchObject {
            package: SearchPackage {
                name: project.key.clone(),
                scope: project.scope.clone(),
                version: newest.version.clone(),
                description: project.description.clone(),
                date: npm_time::format_date_time(&project.updated_at),
                links: SearchLinks { npm: None },
            },
            score: SearchScore::default(),
            search_score: 1.0,
        });
    }

    Ok(ResponseBuilder::ok()
        .json(&SearchResponse {
            objects,
            total,
            time: npm_time::format_date_time(&chrono::Local::now().fixed_offset()),
        })
        .into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_npms_query_string() {
        let query: SearchQuery = serde_urlencoded::from_str("text=lodash&size=5&from=10").unwrap();
        assert_eq!(query.text, "lodash");
        assert_eq!(query.size, Some(5));
        assert_eq!(query.from, Some(10));
    }

    /// npm sends other parameters (`quality`, `popularity`, `maintenance`) that are not used here,
    /// and a missing `text` is a legal "list everything" search.
    #[test]
    fn tolerates_unknown_and_missing_parameters() {
        let query: SearchQuery =
            serde_urlencoded::from_str("text=x&quality=1.0&popularity=0.5").unwrap();
        assert_eq!(query.text, "x");
        let empty: SearchQuery = serde_urlencoded::from_str("").unwrap();
        assert_eq!(empty.text, "");
        assert_eq!(empty.size, None);
    }
}
