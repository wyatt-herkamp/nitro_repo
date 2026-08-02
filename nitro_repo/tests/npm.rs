//! End-to-end npm tests. (#502)
//!
//! These send what `npm publish` and `npm install` send. The scoped-package case is here because it
//! was completely broken — `@scope/pkg` parsed to a scope of `@scope`, and `Display` re-prepended
//! the `@`, so publish stored `@@scope/pkg` while every fetch looked up the correct key. Nothing
//! caught it, because nothing published a scoped package and then asked for it back.

mod common;

use axum::http::{StatusCode, header};
use base64::{Engine, engine::general_purpose::STANDARD};
use common::{TestServer, skip_without_database};
use nitro_repo::seed::artifacts;
use serde_json::json;

/// The packument `npm publish` sends: one version, and the tarball inline as a base64 attachment.
async fn publish(
    server: &TestServer,
    repository: &str,
    name: &str,
    version: &str,
) -> common::TestResponse {
    let tarball = artifacts::npm_tarball(name, version, Some("Integration test")).expect("builds");
    let unscoped = name.rsplit('/').next().unwrap_or(name);
    let file_name = format!("{unscoped}-{version}.tgz");

    let mut manifest = artifacts::package_json(name, version, Some("Integration test"));
    let object = manifest.as_object_mut().unwrap();
    object.insert("_id".to_owned(), json!(format!("{name}@{version}")));
    object.insert(
        "dist".to_owned(),
        json!({
            "integrity": artifacts::integrity(&tarball),
            "shasum": artifacts::shasum(&tarball),
            "tarball": format!("http://localhost:6742/repositories/{repository}/{name}/-/{file_name}"),
        }),
    );
    object.insert("_nodeVersion".to_owned(), json!("20.0.0"));
    object.insert("_npmVersion".to_owned(), json!("10.0.0"));
    object.insert("readme".to_owned(), json!("# test"));
    object.insert("readmeFilename".to_owned(), json!("README.md"));

    let body = json!({
        "_id": name,
        "name": name,
        "versions": { version: manifest },
        "_attachments": {
            file_name: {
                "content_type": "application/octet-stream",
                "data": STANDARD.encode(&tarball),
                "length": tarball.len(),
            }
        },
    });

    server
        .request(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/repositories/{repository}/{name}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("npm-command", "publish")
                .header(
                    header::AUTHORIZATION,
                    format!(
                        "Basic {}",
                        STANDARD.encode(format!(
                            "{}:{}",
                            common::TEST_USERNAME,
                            common::TEST_PASSWORD
                        ))
                    ),
                )
                .body(axum::body::Body::from(body.to_string()))
                .unwrap(),
        )
        .await
}

/// A scoped package must be storable and then fetchable under the *same* key. It was not.
#[tokio::test]
async fn a_scoped_package_round_trips() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database("a_scoped_package_round_trips"));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "npm", "npm")
        .await;

    let response = publish(&server, "local/npm", "@nitro/example", "1.0.0").await;
    assert!(
        response.status.is_success(),
        "publishing a scoped package failed: {} {}",
        response.status,
        response.text
    );

    let response = server.get("/repositories/local/npm/@nitro/example").await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "the packument should come back under the same name it was published as: {}",
        response.text
    );

    let packument = response.json();
    assert_eq!(packument["name"], "@nitro/example");
    assert!(
        packument["versions"]["1.0.0"].is_object(),
        "the published version should be in the packument: {}",
        response.text
    );
    assert_eq!(
        packument["dist-tags"]["latest"], "1.0.0",
        "publishing should set `latest`: {}",
        response.text
    );
}

/// npm names the tarball by the *unscoped* name, and fetches it from a path under the scoped one.
#[tokio::test]
async fn a_published_tarball_can_be_downloaded() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_published_tarball_can_be_downloaded"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "npm", "npm")
        .await;

    publish(&server, "local/npm", "@nitro/example", "1.0.0").await;

    let response = server
        .get("/repositories/local/npm/@nitro/example/-/example-1.0.0.tgz")
        .await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "the tarball should be downloadable: {}",
        response.text
    );

    let expected =
        artifacts::npm_tarball("@nitro/example", "1.0.0", Some("Integration test")).unwrap();
    assert_eq!(response.bytes, expected, "the tarball bytes should match");
}

/// `lodash.merge` is a real package. `validate_name` used to allow only `[a-z0-9_-]`, so a dotted
/// name could not be published at all.
#[tokio::test]
async fn a_dotted_unscoped_name_is_accepted() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database("a_dotted_unscoped_name_is_accepted"));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "npm", "npm")
        .await;

    let response = publish(&server, "local/npm", "nitro.helpers", "0.1.0").await;
    assert!(
        response.status.is_success(),
        "a dotted package name should publish: {} {}",
        response.status,
        response.text
    );

    let response = server.get("/repositories/local/npm/nitro.helpers").await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);
}

/// Re-publishing an existing version used to overwrite the tarball while leaving the database row
/// stale. Immutable published versions are the whole contract of a registry.
#[tokio::test]
async fn republishing_an_existing_version_is_refused() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "republishing_an_existing_version_is_refused"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "npm", "npm")
        .await;

    let first = publish(&server, "local/npm", "@nitro/example", "1.0.0").await;
    assert!(first.status.is_success(), "{}", first.text);

    let second = publish(&server, "local/npm", "@nitro/example", "1.0.0").await;
    assert_eq!(
        second.status,
        StatusCode::CONFLICT,
        "re-publishing the same version should conflict, got {}: {}",
        second.status,
        second.text
    );
}

/// The registry advertises this to `npm ping`, and npm refuses to talk to a registry that 404s it.
#[tokio::test]
async fn the_registry_answers_ping() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database("the_registry_answers_ping"));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "npm", "npm")
        .await;

    let response = server.get("/repositories/local/npm/-/ping").await;
    assert!(
        response.status.is_success(),
        "ping should succeed, got {}: {}",
        response.status,
        response.text
    );
}

/// A publish with a tarball that does not match its own `integrity` is a corrupted upload. It was
/// parsed, stored and never checked.
#[tokio::test]
async fn a_tarball_that_does_not_match_its_integrity_is_refused() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_tarball_that_does_not_match_its_integrity_is_refused"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, "local", "npm", "npm")
        .await;

    let tarball = artifacts::npm_tarball("@nitro/example", "1.0.0", Some("x")).unwrap();
    let mut manifest = artifacts::package_json("@nitro/example", "1.0.0", Some("x"));
    let object = manifest.as_object_mut().unwrap();
    object.insert("_id".to_owned(), json!("@nitro/example@1.0.0"));
    object.insert(
        "dist".to_owned(),
        json!({
            // The integrity of *different* content.
            "integrity": artifacts::integrity(b"not this tarball"),
            "shasum": artifacts::shasum(&tarball),
            "tarball": "http://localhost:6742/repositories/local/npm/@nitro/example/-/example-1.0.0.tgz",
        }),
    );
    object.insert("_nodeVersion".to_owned(), json!("20.0.0"));
    object.insert("_npmVersion".to_owned(), json!("10.0.0"));

    let body = json!({
        "_id": "@nitro/example",
        "name": "@nitro/example",
        "versions": { "1.0.0": manifest },
        "_attachments": {
            "example-1.0.0.tgz": {
                "content_type": "application/octet-stream",
                "data": STANDARD.encode(&tarball),
                "length": tarball.len(),
            }
        },
    });

    let response = server
        .request(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/repositories/local/npm/@nitro/example")
                .header(header::CONTENT_TYPE, "application/json")
                .header("npm-command", "publish")
                .header(
                    header::AUTHORIZATION,
                    format!(
                        "Basic {}",
                        STANDARD.encode(format!(
                            "{}:{}",
                            common::TEST_USERNAME,
                            common::TEST_PASSWORD
                        ))
                    ),
                )
                .body(axum::body::Body::from(body.to_string()))
                .unwrap(),
        )
        .await;

    assert!(
        response.status.is_client_error(),
        "a tarball that does not match its integrity should be refused, got {}: {}",
        response.status,
        response.text
    );
}
