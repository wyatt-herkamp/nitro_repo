//! Database-level tests. (#502)
//!
//! `TestCore` has existed the whole time with exactly one `#[ignore]`d self-test, so no entity
//! method was ever executed against Postgres. Both of the defects these cover were invisible to the
//! unit tests: one because it only appears with more than one row present, the other because it
//! only appears when a value is decoded back out of a real column.
//!
//! Like the server's integration tests, these skip when there is no database, unless
//! `NITRO_TESTS_REQUIRE_DB=1` — which CI sets, so a skip there is a failure.

use nr_core::{
    database::entities::project::{
        NewProject,
        versions::{DBProjectVersion, NewVersion, UpdateProjectVersion},
    },
    repository::project::{ReleaseType, VersionData},
    testing::TestCore,
};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Serializes the migration run.
///
/// `TestCore::new` runs the migrations every time. These tests run in parallel against one shared
/// database, so several of them tried to create the same types at once and collided on
/// `pg_type_typname_nsp_index`. Only the setup is serialized — the pool itself must stay per-test,
/// because each `#[tokio::test]` has its own runtime and a pool created in one dies when that
/// runtime shuts down.
static MIGRATIONS: Mutex<()> = Mutex::const_new(());

/// `None` when there is no database to use.
async fn core(function_path: &str) -> Option<PgPool> {
    let result = {
        let _guard = MIGRATIONS.lock().await;
        TestCore::new(function_path.to_owned()).await
    };

    match result {
        Ok((core, _entry)) => Some(core.db),
        Err(error) => {
            if std::env::var("NITRO_TESTS_REQUIRE_DB").as_deref() == Ok("1") {
                panic!(
                    "{function_path} needs a database and NITRO_TESTS_REQUIRE_DB=1 was set: {error}"
                );
            }
            eprintln!("skipping {function_path}: no test database ({error})");
            None
        }
    }
}

/// A storage and a repository for a project to hang off.
///
/// Written as raw SQL because creating a repository lives in the server crate, not here — this only
/// needs the rows to satisfy the foreign keys.
async fn repository(db: &PgPool) -> Uuid {
    let unique = Uuid::new_v4();
    let storage_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO storages (storage_type, name, config)
           VALUES ('Local', $1, '{"path": "/tmp/nitro-core-tests"}'::jsonb)
           RETURNING id"#,
    )
    .bind(format!("test-storage-{unique}"))
    .fetch_one(db)
    .await
    .expect("the test storage should insert");

    sqlx::query_scalar(
        r#"INSERT INTO repositories (id, storage_id, name, repository_type, active)
           VALUES ($1, $2, $3, 'maven', true)
           RETURNING id"#,
    )
    .bind(unique)
    .bind(storage_id)
    .bind(format!("test-repo-{unique}"))
    .fetch_one(db)
    .await
    .expect("the test repository should insert")
}

async fn project(db: &PgPool, repository: Uuid) -> Uuid {
    let unique = Uuid::new_v4();
    NewProject {
        scope: Some("dev.kingtux".to_owned()),
        project_key: format!("dev.kingtux:test-{unique}"),
        name: format!("test-{unique}"),
        description: None,
        repository,
        storage_path: format!("dev/kingtux/test-{unique}"),
    }
    .insert(db)
    .await
    .expect("the project should insert")
    .id
}

async fn version(db: &PgPool, project_id: Uuid, version: &str) -> Uuid {
    NewVersion {
        project_id,
        version: version.to_owned(),
        release_type: ReleaseType::Stable,
        version_path: format!("dev/kingtux/test/{version}"),
        publisher: None,
        version_page: None,
        extra: VersionData::default(),
    }
    .insert(db)
    .await
    .expect("the version should insert")
    .id
}

/// Reads one version straight out of the table, bypassing every helper.
///
/// The point of the unfiltered-UPDATE regression is that a row's *id* was rewritten, so looking it
/// up by anything other than its id would not notice.
async fn version_by_id(db: &PgPool, id: Uuid) -> Option<DBProjectVersion> {
    sqlx::query_as("SELECT * FROM project_versions WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .expect("query")
}

/// The regression for the unfiltered UPDATE.
///
/// `UpdateProjectVersion::update` passed the target id to `set` rather than `filter`, producing an
/// `UPDATE ... SET id = $1` with no `WHERE` at all — it rewrote the primary key of *every* version
/// in the table. With one row it silently looked fine, which is why it survived: this needs two.
#[tokio::test]
async fn updating_one_version_leaves_the_others_alone() {
    let Some(db) = core("database::updating_one_version_leaves_the_others_alone").await else {
        return;
    };

    let repository = repository(&db).await;
    let project_id = project(&db, repository).await;
    let first = version(&db, project_id, "1.0.0").await;
    let second = version(&db, project_id, "2.0.0").await;

    UpdateProjectVersion {
        release_type: Some(ReleaseType::Unknown),
        ..Default::default()
    }
    .update(first, &db)
    .await
    .expect("the update should succeed");

    let updated = version_by_id(&db, first)
        .await
        .expect("the updated version should still exist under its own id");
    assert_eq!(updated.release_type, ReleaseType::Unknown);
    assert_eq!(updated.version, "1.0.0");

    let untouched = version_by_id(&db, second)
        .await
        .expect("the other version should still exist — an unfiltered UPDATE rewrites its id");
    assert_eq!(
        untouched.release_type,
        ReleaseType::Stable,
        "updating one version must not change another"
    );
    assert_eq!(untouched.version, "2.0.0");
}

/// The regression for the column type mismatch.
///
/// `project_versions.release_type` was VARCHAR(255) while `ReleaseType` declares
/// `#[sqlx(type_name = "TEXT")]`. sqlx checks the column's type OID when decoding, so *reading* a
/// version failed even though writing one worked. Nothing that listed Maven versions returned any,
/// and the deploy still reported success.
#[tokio::test]
async fn every_release_type_survives_a_round_trip() {
    let Some(db) = core("database::every_release_type_survives_a_round_trip").await else {
        return;
    };

    let repository = repository(&db).await;
    let project_id = project(&db, repository).await;

    for (index, release_type) in [
        ReleaseType::Stable,
        ReleaseType::Beta,
        ReleaseType::Alpha,
        ReleaseType::Snapshot,
        ReleaseType::ReleaseCandidate,
        ReleaseType::Unknown,
    ]
    .into_iter()
    .enumerate()
    {
        let version_string = format!("1.0.{index}");
        NewVersion {
            project_id,
            version: version_string.clone(),
            release_type: release_type.clone(),
            version_path: format!("dev/kingtux/test/{version_string}"),
            publisher: None,
            version_page: None,
            extra: VersionData::default(),
        }
        .insert(&db)
        .await
        .expect("the version should insert");

        // The decode is the point. Inserting worked before the fix; reading back did not.
        let found = DBProjectVersion::find_by_version_and_project(&version_string, project_id, &db)
            .await
            .unwrap_or_else(|error| {
                panic!("reading back a {release_type:?} version failed: {error}")
            })
            .expect("the version should be found");

        assert_eq!(found.release_type, release_type);
    }
}

/// Versions come back newest-first. Without an `ORDER BY` this is whatever Postgres happens to
/// return, which is what made npm's `dist-tags.latest` nondeterministic.
#[tokio::test]
async fn versions_are_returned_in_a_defined_order() {
    let Some(db) = core("database::versions_are_returned_in_a_defined_order").await else {
        return;
    };

    let repository = repository(&db).await;
    let project_id = project(&db, repository).await;

    version(&db, project_id, "1.0.0").await;
    version(&db, project_id, "1.1.0").await;
    version(&db, project_id, "2.0.0").await;

    let first = DBProjectVersion::get_all_versions(project_id, &db)
        .await
        .expect("query");
    let second = DBProjectVersion::get_all_versions(project_id, &db)
        .await
        .expect("query");

    assert_eq!(first.len(), 3);
    let first: Vec<&str> = first.iter().map(|value| value.version.as_str()).collect();
    let second: Vec<&str> = second.iter().map(|value| value.version.as_str()).collect();
    assert_eq!(
        first, second,
        "two identical queries must return the same order"
    );
}
