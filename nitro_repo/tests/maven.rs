//! End-to-end Maven tests. (#502)
//!
//! These drive the real router with the requests `mvn deploy` and `mvn dependency:get` send. Every
//! one of them covers something that was broken and is now not — most of Phase 4 had no test that
//! executed against a database, which is how the `release_type` column mismatch survived.

mod common;

use axum::http::StatusCode;
use common::{TestServer, skip_without_database};
use nitro_repo::seed::{artifacts, config::MavenProject};

fn project(repository: &str) -> MavenProject {
    MavenProject {
        repository: repository.to_owned(),
        group_id: "dev.kingtux".to_owned(),
        artifact_id: "tms".to_owned(),
        versions: vec![],
        description: Some("Integration test artifact".to_owned()),
        classifiers: vec![],
        dependencies: vec![],
    }
}

/// Deploys one version the way Maven does: the POM, the jar, and a checksum beside each.
async fn deploy(server: &TestServer, repository: &str, version: &str) {
    let project = project(repository);
    let base = format!("/repositories/{repository}/dev/kingtux/tms/{version}/tms-{version}");

    let pom = artifacts::pom(&project, version).into_bytes();
    let response = server
        .put_with_basic(&format!("{base}.pom"), pom.clone(), "application/xml")
        .await;
    assert!(
        response.status.is_success(),
        "deploying the POM failed: {} {}",
        response.status,
        response.text
    );

    let sha1 = artifacts::sha1_hex(&pom).into_bytes();
    let response = server
        .put_with_basic(&format!("{base}.pom.sha1"), sha1, "text/plain")
        .await;
    assert!(
        response.status.is_success(),
        "deploying the POM checksum failed: {} {}",
        response.status,
        response.text
    );

    let jar = artifacts::jar(&project, version, None).expect("builds");
    let response = server
        .put_with_basic(
            &format!("{base}.jar"),
            jar.clone(),
            "application/java-archive",
        )
        .await;
    assert!(
        response.status.is_success(),
        "deploying the jar failed: {} {}",
        response.status,
        response.text
    );

    let sha1 = artifacts::sha1_hex(&jar).into_bytes();
    let response = server
        .put_with_basic(&format!("{base}.jar.sha1"), sha1, "text/plain")
        .await;
    assert!(
        response.status.is_success(),
        "deploying the jar checksum failed: {} {}",
        response.status,
        response.text
    );
}

#[tokio::test]
async fn a_deployed_artifact_can_be_fetched_back() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_deployed_artifact_can_be_fetched_back"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    deploy(&server, "local/releases", "1.0.0").await;

    let response = server
        .get("/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.jar")
        .await;
    assert_eq!(response.status, StatusCode::OK);

    let expected = artifacts::jar(&project("local/releases"), "1.0.0", None).unwrap();
    assert_eq!(
        response.bytes, expected,
        "the bytes served back should be the bytes deployed"
    );
}

/// The defect that made this necessary: `release_type` was VARCHAR while the Rust type declared
/// TEXT, so every read of a version failed and this list was always empty — while the deploy itself
/// reported success.
#[tokio::test]
async fn a_deployed_version_is_registered_in_the_database() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_deployed_version_is_registered_in_the_database"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    deploy(&server, "local/releases", "1.0.0").await;
    deploy(&server, "local/releases", "1.1.0").await;

    // Search reads through `project_versions`, which is the table whose `release_type` column could
    // not be decoded — so this is the assertion the original defect would fail.
    let response = server.get_as("/api/search?text=tms", &session).await;
    assert!(
        response.status.is_success(),
        "search failed: {} {}",
        response.status,
        response.text
    );

    let results = response.json();
    let results = results["results"].as_array().expect("results array");
    assert_eq!(
        results.len(),
        2,
        "both deployed versions should be registered, got: {}",
        response.text
    );

    let versions: Vec<&str> = results
        .iter()
        .filter_map(|value| value["version"].as_str())
        .collect();
    assert!(versions.contains(&"1.0.0"), "{versions:?}");
    assert!(versions.contains(&"1.1.0"), "{versions:?}");

    for result in results {
        assert_eq!(result["repository"], "releases", "{result}");
    }
    let _ = repository;
}

/// `maven-metadata.xml` is generated, never stored. Nothing uploads it, so if the generator is
/// wrong the only symptom is that clients cannot resolve a version range.
#[tokio::test]
async fn maven_metadata_is_generated_from_the_deployed_versions() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "maven_metadata_is_generated_from_the_deployed_versions"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    deploy(&server, "local/releases", "1.0.0").await;
    deploy(&server, "local/releases", "1.1.0").await;

    let response = server
        .get("/repositories/local/releases/dev/kingtux/tms/maven-metadata.xml")
        .await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "maven-metadata.xml should be generated: {}",
        response.text
    );

    let body = &response.text;
    assert!(body.contains("<groupId>dev.kingtux</groupId>"), "{body}");
    assert!(body.contains("<artifactId>tms</artifactId>"), "{body}");
    assert!(body.contains("<version>1.0.0</version>"), "{body}");
    assert!(body.contains("<version>1.1.0</version>"), "{body}");
    // `release` is the newest non-snapshot, which is what a client resolving RELEASE reads.
    assert!(body.contains("<release>1.1.0</release>"), "{body}");
}

/// A checksum that was never uploaded still has to be served — Maven asks for one after every
/// download and treats a 404 as a corrupt artifact.
#[tokio::test]
async fn checksums_are_served_for_files_that_were_never_uploaded_as_checksums() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "checksums_are_served_for_files_that_were_never_uploaded_as_checksums"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    // Deployed *without* an md5 beside it.
    let project = project("local/releases");
    let jar = artifacts::jar(&project, "1.0.0", None).unwrap();
    server
        .put_with_basic(
            "/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.jar",
            jar.clone(),
            "application/java-archive",
        )
        .await;

    let response = server
        .get("/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.jar.md5")
        .await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "an md5 should be generated on demand: {}",
        response.text
    );
    assert_eq!(response.text.trim(), artifacts::md5_hex(&jar));
}

/// An uploaded checksum that does not match the artifact means the upload was corrupted in transit.
/// Accepting it produces a repository whose contents cannot be validated.
#[tokio::test]
async fn a_mismatched_checksum_is_rejected() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database("a_mismatched_checksum_is_rejected"));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    let project = project("local/releases");
    let jar = artifacts::jar(&project, "1.0.0", None).unwrap();
    server
        .put_with_basic(
            "/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.jar",
            jar,
            "application/java-archive",
        )
        .await;

    let response = server
        .put_with_basic(
            "/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.jar.sha1",
            b"0000000000000000000000000000000000000000".to_vec(),
            "text/plain",
        )
        .await;

    assert!(
        response.status.is_client_error(),
        "a checksum that does not match should be refused, got {}: {}",
        response.status,
        response.text
    );
}

/// Anonymous writes are how a repository ends up with artifacts nobody deployed.
#[tokio::test]
async fn an_anonymous_deploy_is_refused() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database("an_anonymous_deploy_is_refused"));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    let project = project("local/releases");
    let response = server
        .request(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.pom")
                .header(axum::http::header::CONTENT_TYPE, "application/xml")
                .body(axum::body::Body::from(artifacts::pom(&project, "1.0.0")))
                .unwrap(),
        )
        .await;

    assert!(
        response.status == StatusCode::UNAUTHORIZED || response.status == StatusCode::FORBIDDEN,
        "an unauthenticated deploy should be refused, got {}",
        response.status
    );
}

/// Deploys a POM carrying a `<name>` — a human title, which is not a coordinate.
async fn deploy_named_pom(server: &TestServer, repository: &str, title: &str) {
    let pom = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>dev.kingtux</groupId>
    <artifactId>titled</artifactId>
    <version>1.0.0</version>
    <name>{title}</name>
</project>"#
    );
    let response = server
        .put_with_basic(
            &format!("/repositories/{repository}/dev/kingtux/titled/1.0.0/titled-1.0.0.pom"),
            pom.into_bytes(),
            "application/xml",
        )
        .await;
    assert!(
        response.status.is_success(),
        "deploying the POM failed: {} {}",
        response.status,
        response.text
    );
}

/// What the project page reads off a Maven project.
///
/// Three things were wrong at once. The API serialized the database row directly, so it answered
/// with `key` and `path` while the page reads `project_key` and `storage_path`, and it carried no
/// version at all — so the Gradle line rendered as `undefined:latest`. And ingest stored the POM's
/// `<name>` as the project name, so the field the page labels "Artifact Id" held a human title.
#[tokio::test]
async fn the_project_api_returns_the_coordinate_and_the_latest_version() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "the_project_api_returns_the_coordinate_and_the_latest_version"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    deploy(&server, "local/releases", "1.0.0").await;
    deploy(&server, "local/releases", "1.1.0").await;
    deploy(&server, "local/releases", "2.0.0-SNAPSHOT").await;

    let response = server
        .get_as(
            &format!("/api/project/by-key/{repository}/dev.kingtux:tms"),
            &session,
        )
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);

    let project = response.json();
    assert_eq!(project["project_key"], "dev.kingtux:tms", "{project}");
    assert_eq!(project["scope"], "dev.kingtux", "{project}");
    assert_eq!(
        project["name"], "tms",
        "`name` is the artifactId — it is what the page labels \"Artifact Id\": {project}"
    );
    assert_eq!(
        project["storage_path"], "dev/kingtux/tms",
        "the page links straight into browse with this: {project}"
    );
    assert_eq!(
        project["latest_release"], "1.1.0",
        "a snapshot is not a release: {project}"
    );
    assert_eq!(project["latest_pre_release"], "2.0.0-SNAPSHOT", "{project}");
}

/// A POM's `<name>` is a title, not a coordinate — nobody can depend on "Totally Not An Artifact
/// Id", so it must not end up in the field the dependency snippets use as the artifactId.
#[tokio::test]
async fn a_poms_name_element_does_not_become_the_artifact_id() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_poms_name_element_does_not_become_the_artifact_id"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    deploy_named_pom(&server, "local/releases", "Totally Not An Artifact Id").await;

    let response = server
        .get_as(
            &format!("/api/project/by-key/{repository}/dev.kingtux:titled"),
            &session,
        )
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);
    assert_eq!(response.json()["name"], "titled", "{}", response.text);
}
