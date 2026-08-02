//! Authorization tests. (#502)
//!
//! Three separate information leaks were found by reading code during this work: the repository
//! list returned every private repository to anonymous callers, all three project routes took an
//! `auth` argument and never used it, and `get_config` hardcoded `Visibility::Private`. None of
//! them had a test. These are the tests.
//!
//! Every case here asserts a *refusal*. A test that only checks the happy path cannot tell the
//! difference between working authorization and none at all.

mod common;

use axum::http::StatusCode;
use common::{TestServer, skip_without_database};
use serde_json::json;

/// Makes a repository private. Creation does not take a visibility, so it is a follow-up.
async fn make_private(server: &TestServer, session: &str, repository: &str) {
    let response = server
        .put_json_as(
            &format!("/api/repository/{repository}"),
            json!({ "visibility": "Private" }),
            session,
        )
        .await;
    assert!(
        response.status.is_success(),
        "could not make the repository private: {} {}",
        response.status,
        response.text
    );
}

/// The list used to return every repository regardless of who was asking, so a private
/// repository's name, storage and type were readable by anyone who could reach the instance.
#[tokio::test]
async fn a_private_repository_is_not_listed_anonymously() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_private_repository_is_not_listed_anonymously"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "secret", "maven")
        .await;
    make_private(&server, &session, &repository).await;

    let response = server.get("/api/repository/list").await;
    assert!(
        response.status.is_success(),
        "the list itself should still answer: {}",
        response.text
    );
    assert!(
        !response.text.contains("secret"),
        "a private repository must not appear in the anonymous list: {}",
        response.text
    );

    // The owner still sees it, or the filter is simply broken rather than correct.
    let response = server.get_as("/api/repository/list", &session).await;
    assert!(
        response.text.contains("secret"),
        "an admin should still see their own private repository: {}",
        response.text
    );
}

/// All three project routes took an `auth` argument and never read it, so every project and version
/// in a private repository was readable by anyone who could guess an id.
#[tokio::test]
async fn projects_in_a_private_repository_are_not_readable_anonymously() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "projects_in_a_private_repository_are_not_readable_anonymously"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "secret", "maven")
        .await;

    // Deployed while the repository is still public, so the project exists before it is locked.
    let project = nitro_repo::seed::config::MavenProject {
        repository: "local/secret".to_owned(),
        group_id: "dev.kingtux".to_owned(),
        artifact_id: "private-lib".to_owned(),
        versions: vec![],
        description: None,
        classifiers: vec![],
        dependencies: vec![],
    };
    let pom = nitro_repo::seed::artifacts::pom(&project, "1.0.0");
    server
        .put_with_basic(
            "/repositories/local/secret/dev/kingtux/private-lib/1.0.0/private-lib-1.0.0.pom",
            pom.into_bytes(),
            "application/xml",
        )
        .await;

    let projects = server
        .get_as("/api/search?text=private-lib", &session)
        .await;
    let projects = projects.json();
    let project_id = projects["results"][0]["project_id"]
        .as_str()
        .expect("the deployed project should be findable while public")
        .to_owned();

    make_private(&server, &session, &repository).await;

    let response = server.get(&format!("/api/project/{project_id}")).await;
    assert!(
        response.status == StatusCode::NOT_FOUND
            || response.status == StatusCode::UNAUTHORIZED
            || response.status == StatusCode::FORBIDDEN,
        "a project in a private repository must not be readable anonymously, got {}: {}",
        response.status,
        response.text
    );

    let response = server
        .get(&format!("/api/project/{project_id}/versions"))
        .await;
    assert!(
        response.status == StatusCode::NOT_FOUND
            || response.status == StatusCode::UNAUTHORIZED
            || response.status == StatusCode::FORBIDDEN,
        "versions in a private repository must not be readable anonymously, got {}: {}",
        response.status,
        response.text
    );
}

/// Search filters by visibility after the fact. A private repository's contents leaking through it
/// would be the same defect in a different route.
#[tokio::test]
async fn search_does_not_return_private_repositories_anonymously() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "search_does_not_return_private_repositories_anonymously"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "secret", "maven")
        .await;

    let project = nitro_repo::seed::config::MavenProject {
        repository: "local/secret".to_owned(),
        group_id: "dev.kingtux".to_owned(),
        artifact_id: "hidden-lib".to_owned(),
        versions: vec![],
        description: None,
        classifiers: vec![],
        dependencies: vec![],
    };
    let pom = nitro_repo::seed::artifacts::pom(&project, "1.0.0");
    server
        .put_with_basic(
            "/repositories/local/secret/dev/kingtux/hidden-lib/1.0.0/hidden-lib-1.0.0.pom",
            pom.into_bytes(),
            "application/xml",
        )
        .await;

    make_private(&server, &session, &repository).await;

    let response = server.get("/api/search?text=hidden-lib").await;
    assert!(response.status.is_success(), "{}", response.text);
    let results = response.json();
    assert_eq!(
        results["results"].as_array().map(Vec::len),
        Some(0),
        "search must not surface a private repository's contents: {}",
        response.text
    );

    let response = server.get_as("/api/search?text=hidden-lib", &session).await;
    let results = response.json();
    assert_eq!(
        results["results"].as_array().map(Vec::len),
        Some(1),
        "the owner should still find it: {}",
        response.text
    );
}

/// Reading a file out of a private repository is the leak that matters most — it is the artifact
/// itself, not metadata about it.
#[tokio::test]
async fn files_in_a_private_repository_are_not_downloadable_anonymously() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "files_in_a_private_repository_are_not_downloadable_anonymously"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "secret", "maven")
        .await;

    let project = nitro_repo::seed::config::MavenProject {
        repository: "local/secret".to_owned(),
        group_id: "dev.kingtux".to_owned(),
        artifact_id: "private-lib".to_owned(),
        versions: vec![],
        description: None,
        classifiers: vec![],
        dependencies: vec![],
    };
    let jar = nitro_repo::seed::artifacts::jar(&project, "1.0.0", None).unwrap();
    let path = "/repositories/local/secret/dev/kingtux/private-lib/1.0.0/private-lib-1.0.0.jar";
    server
        .put_with_basic(path, jar, "application/java-archive")
        .await;

    make_private(&server, &session, &repository).await;

    let response = server.get(path).await;
    assert!(
        !response.status.is_success(),
        "a file in a private repository must not be downloadable anonymously, got {}",
        response.status
    );
}

/// Administration must be refused to callers who are not administrators. Without a check here, the
/// only thing standing between a signed-in user and creating storages is the frontend hiding a link.
#[tokio::test]
async fn administration_is_refused_to_anonymous_callers() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "administration_is_refused_to_anonymous_callers"
        ));
        return;
    };
    server.install().await;

    let response = server.get("/api/storage/list").await;
    assert!(
        response.status == StatusCode::UNAUTHORIZED || response.status == StatusCode::FORBIDDEN,
        "listing storages anonymously should be refused, got {}: {}",
        response.status,
        response.text
    );

    let response = server
        .post_json(
            "/api/storage/new/Local",
            json!({
                "name": "sneaky",
                "config": { "type": "Local", "settings": { "path": "/tmp/sneaky" } },
            }),
        )
        .await;
    assert!(
        response.status == StatusCode::UNAUTHORIZED || response.status == StatusCode::FORBIDDEN,
        "creating a storage anonymously should be refused, got {}: {}",
        response.status,
        response.text
    );

    let response = server.get("/api/user-management/list").await;
    assert!(
        response.status == StatusCode::UNAUTHORIZED || response.status == StatusCode::FORBIDDEN,
        "listing users anonymously should be refused, got {}: {}",
        response.status,
        response.text
    );
}

/// Installing twice would hand an attacker a second administrator on any reachable instance.
#[tokio::test]
async fn the_instance_cannot_be_installed_twice() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "the_instance_cannot_be_installed_twice"
        ));
        return;
    };
    server.install().await;

    let response = server
        .post_json(
            "/api/install",
            json!({
                "user": {
                    "username": "second_admin",
                    "email": "second@example.com",
                    "name": "Second Admin",
                    "password": "TestPassword1",
                }
            }),
        )
        .await;

    assert!(
        !response.status.is_success(),
        "a second install should be refused, got {}: {}",
        response.status,
        response.text
    );
}
