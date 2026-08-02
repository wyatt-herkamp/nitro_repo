//! End-to-end tests for custom domains.
//!
//! A repository can be given a hostname; a request arriving on that host is served by that
//! repository, with the whole request path taken as the artifact path. These drive the real router,
//! so the `Host` header is the only thing distinguishing a domain-routed request from any other.

mod common;

use axum::http::{StatusCode, header};
use base64::{Engine, engine::general_purpose::STANDARD};
use common::{TestServer, skip_without_database};
use nitro_repo::seed::{artifacts, config::MavenProject};
use serde_json::json;

const HOST: &str = "maven.example.com";

fn project() -> MavenProject {
    MavenProject {
        repository: "local/releases".to_owned(),
        group_id: "dev.kingtux".to_owned(),
        artifact_id: "tms".to_owned(),
        versions: vec![],
        description: Some("Integration test artifact".to_owned()),
        classifiers: vec![],
        dependencies: vec![],
    }
}

const ARTIFACT_PATH: &str = "/dev/kingtux/tms/1.0.0/tms-1.0.0.jar";

/// Installs, signs in, creates `local/releases` and deploys one jar into it over the path route.
async fn maven_server() -> Option<(TestServer, String, String, Vec<u8>)> {
    let server = TestServer::start().await?;
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "releases", "maven")
        .await;

    let jar = artifacts::jar(&project(), "1.0.0", None).expect("builds");
    let response = server
        .put_with_basic(
            &format!("/repositories/local/releases{ARTIFACT_PATH}"),
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

    Some((server, session, repository, jar))
}

#[tokio::test]
async fn a_registered_host_serves_the_repository() {
    let Some((server, session, repository, jar)) = maven_server().await else {
        assert!(skip_without_database(
            "a_registered_host_serves_the_repository"
        ));
        return;
    };
    let response = server.add_hostname(&session, &repository, HOST).await;
    assert_eq!(
        response.status,
        StatusCode::CREATED,
        "registering a hostname failed: {}",
        response.text
    );

    let response = server.get_with_host(HOST, ARTIFACT_PATH).await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "the artifact should be served on the custom domain: {}",
        response.text
    );
    assert_eq!(
        response.bytes, jar,
        "the custom domain should serve the same bytes as the path route"
    );
}

#[tokio::test]
async fn an_unregistered_host_does_not_reach_the_repository() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database(
            "an_unregistered_host_does_not_reach_the_repository"
        ));
        return;
    };
    server.add_hostname(&session, &repository, HOST).await;

    let response = server
        .get_with_host("nope.example.com", ARTIFACT_PATH)
        .await;
    assert_ne!(
        response.status,
        StatusCode::OK,
        "an unregistered host should not serve artifacts: {}",
        response.text
    );
}

/// DNS is case-insensitive and a browser or client may include the port it connected to.
#[tokio::test]
async fn the_host_is_matched_ignoring_case_and_port() {
    let Some((server, session, repository, jar)) = maven_server().await else {
        assert!(skip_without_database(
            "the_host_is_matched_ignoring_case_and_port"
        ));
        return;
    };
    server.add_hostname(&session, &repository, HOST).await;

    let response = server
        .get_with_host("MAVEN.Example.com:8443", ARTIFACT_PATH)
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);
    assert_eq!(response.bytes, jar);
}

/// Anyone can send `X-Forwarded-Host`. Honouring it without an operator saying their proxy sets it
/// would let a caller choose which repository their request lands in.
#[tokio::test]
async fn x_forwarded_host_is_not_trusted_by_default() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database(
            "x_forwarded_host_is_not_trusted_by_default"
        ));
        return;
    };
    server.add_hostname(&session, &repository, HOST).await;

    let response = server
        .get_with_headers(
            ARTIFACT_PATH,
            &[("host", "nope.example.com"), ("x-forwarded-host", HOST)],
        )
        .await;
    assert_ne!(
        response.status,
        StatusCode::OK,
        "a spoofed X-Forwarded-Host should not route into a repository: {}",
        response.text
    );
}

#[tokio::test]
async fn a_duplicate_hostname_is_a_conflict() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database("a_duplicate_hostname_is_a_conflict"));
        return;
    };
    let response = server.add_hostname(&session, &repository, HOST).await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.text);

    let response = server.add_hostname(&session, &repository, HOST).await;
    assert_eq!(
        response.status,
        StatusCode::CONFLICT,
        "the same hostname should not be registerable twice: {}",
        response.text
    );
}

/// The harness sets `app_url` to `http://localhost:6742/`. Claiming that host would route the web
/// UI into a repository and leave the instance with no way to undo it.
#[tokio::test]
async fn the_instances_own_hostname_is_refused() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database(
            "the_instances_own_hostname_is_refused"
        ));
        return;
    };
    let response = server
        .add_hostname(&session, &repository, "localhost")
        .await;
    assert_eq!(
        response.status,
        StatusCode::CONFLICT,
        "the instance's own host should not be claimable: {}",
        response.text
    );
}

#[tokio::test]
async fn deleting_a_hostname_stops_routing_it() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database(
            "deleting_a_hostname_stops_routing_it"
        ));
        return;
    };
    let created = server.add_hostname(&session, &repository, HOST).await;
    let hostname_id = created.json()["id"].as_i64().expect("the created id");

    assert_eq!(
        server.get_with_host(HOST, ARTIFACT_PATH).await.status,
        StatusCode::OK
    );

    let response = server
        .delete_as(
            &format!("/api/repository/{repository}/hostnames/{hostname_id}"),
            &session,
        )
        .await;
    assert_eq!(response.status, StatusCode::NO_CONTENT, "{}", response.text);

    assert_ne!(
        server.get_with_host(HOST, ARTIFACT_PATH).await.status,
        StatusCode::OK,
        "a removed domain should stop routing"
    );
}

/// The database rows go by `ON DELETE CASCADE`; the in-memory index has to be cleared too, or the
/// host keeps resolving and the name can never be reused.
#[tokio::test]
async fn deleting_the_repository_releases_its_hostname() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database(
            "deleting_the_repository_releases_its_hostname"
        ));
        return;
    };
    server.add_hostname(&session, &repository, HOST).await;

    let response = server
        .delete_as(&format!("/api/repository/{repository}"), &session)
        .await;
    assert_eq!(response.status, StatusCode::NO_CONTENT, "{}", response.text);

    assert_ne!(
        server.get_with_host(HOST, ARTIFACT_PATH).await.status,
        StatusCode::OK,
        "the host should stop routing once its repository is gone"
    );

    let replacement = server
        .create_repository(&session, "local", "releases2", "maven")
        .await;
    let response = server.add_hostname(&session, &replacement, HOST).await;
    assert_eq!(
        response.status,
        StatusCode::CREATED,
        "the freed hostname should be registerable again: {}",
        response.text
    );
}

/// A directory request without a trailing slash is redirected with a *relative* `Location`, which
/// is what lets the same code serve both the `/repositories/...` prefix and a bare domain.
#[tokio::test]
async fn a_directory_request_redirects_to_the_slash_form() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database(
            "a_directory_request_redirects_to_the_slash_form"
        ));
        return;
    };
    server.add_hostname(&session, &repository, HOST).await;

    let response = server.get_with_host(HOST, "/dev/kingtux/tms/1.0.0").await;
    assert_eq!(
        response.status,
        StatusCode::MOVED_PERMANENTLY,
        "a directory should redirect to its slash form: {}",
        response.text
    );
    assert_eq!(
        response.headers.get(header::LOCATION).unwrap(),
        "1.0.0/",
        "the redirect should be relative so it works under any prefix"
    );
}

/// The load-bearing assertion for the routing decision: `/api`, `/badge` and `/repositories` are
/// matched before the host-aware fallback, on every host. Without this a custom domain would take
/// the admin API away from anyone browsing through it.
#[tokio::test]
async fn the_api_still_answers_on_a_registered_host() {
    let Some((server, session, repository, _)) = maven_server().await else {
        assert!(skip_without_database(
            "the_api_still_answers_on_a_registered_host"
        ));
        return;
    };
    server.add_hostname(&session, &repository, HOST).await;

    let response = server.get_with_host(HOST, "/api/info").await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "the API should answer on a custom domain: {}",
        response.text
    );
    assert_eq!(response.json()["name"], "Integration tests");

    // And so should the canonical artifact URL.
    let response = server
        .get_with_host(
            HOST,
            &format!("/repositories/local/releases{ARTIFACT_PATH}"),
        )
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);
}

/// npm rewrites `dist.tarball` to the registry it publishes to, so a domain-routed publish sends a
/// URL with no storage or repository in the path. Rejecting that refused every publish over a
/// custom domain.
#[tokio::test]
async fn an_npm_package_round_trips_over_a_custom_domain() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "an_npm_package_round_trips_over_a_custom_domain"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, "local", "npm", "npm")
        .await;

    let host = "npm.example.com";
    let response = server.add_hostname(&session, &repository, host).await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.text);

    let name = "@nitro/example";
    let version = "1.0.0";
    let tarball = artifacts::npm_tarball(name, version, Some("Integration test")).expect("builds");
    let file_name = format!("example-{version}.tgz");

    let mut manifest = artifacts::package_json(name, version, Some("Integration test"));
    let object = manifest.as_object_mut().unwrap();
    object.insert("_id".to_owned(), json!(format!("{name}@{version}")));
    object.insert(
        "dist".to_owned(),
        json!({
            "integrity": artifacts::integrity(&tarball),
            "shasum": artifacts::shasum(&tarball),
            // What npm actually sends when the registry is a bare domain.
            "tarball": format!("https://{host}/{name}/-/{file_name}"),
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
            file_name.clone(): {
                "content_type": "application/octet-stream",
                "data": STANDARD.encode(&tarball),
                "length": tarball.len(),
            }
        },
    });

    let credentials = STANDARD.encode(format!(
        "{}:{}",
        common::TEST_USERNAME,
        common::TEST_PASSWORD
    ));
    let response = server
        .request(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/{name}"))
                .header(header::HOST, host)
                .header(header::CONTENT_TYPE, "application/json")
                .header("npm-command", "publish")
                .header(header::AUTHORIZATION, format!("Basic {credentials}"))
                .body(axum::body::Body::from(body.to_string()))
                .unwrap(),
        )
        .await;
    assert!(
        response.status.is_success(),
        "publishing over a custom domain failed: {} {}",
        response.status,
        response.text
    );

    let response = server
        .get_with_host(host, &format!("/{name}/-/{file_name}"))
        .await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "the tarball should be downloadable over the custom domain: {}",
        response.text
    );
    assert_eq!(response.bytes, tarball);
}
