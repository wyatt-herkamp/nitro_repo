//! Cargo registry integration tests.
//!
//! Drives the real router with the requests `cargo publish`, `cargo add` and `cargo yank` send.
//! No `cargo` binary is invoked: the bytes are built here so a test can assert on the exact frame
//! layout and header shape a real client produces, including the ones a passing `cargo publish`
//! would never let us see.

mod common;

use axum::http::StatusCode;
use common::{TestServer, skip_without_database};
use nitro_repo::seed::artifacts;
use serde_json::{Value, json};

const STORAGE: &str = "local";
const REPOSITORY: &str = "crates";

fn base() -> String {
    format!("/repositories/{STORAGE}/{REPOSITORY}")
}

fn metadata(name: &str, version: &str) -> Value {
    json!({
        "name": name,
        "vers": version,
        "deps": [],
        "features": {},
        "authors": ["Test Admin"],
        "description": "A crate published by the integration tests",
        "documentation": null,
        "homepage": null,
        "readme": null,
        "readme_file": null,
        "keywords": [],
        "categories": [],
        "license": "MIT",
        "license_file": null,
        "repository": null,
        "links": null,
    })
}

struct Registry {
    server: TestServer,
    token: String,
}

impl Registry {
    async fn start() -> Option<Self> {
        let server = TestServer::start().await?;
        server.install().await;
        let session = server.sign_in().await;
        let repository = server
            .create_repository(&session, STORAGE, REPOSITORY, "cargo")
            .await;
        let token = server
            .create_repository_token(&session, &repository, &["Read", "Write", "Edit"])
            .await;
        Some(Self { server, token })
    }

    /// What `cargo publish` sends: `PUT /api/v1/crates/new` with the two-frame body and a bare
    /// token.
    async fn publish(&self, name: &str, version: &str) -> common::TestResponse {
        let crate_file = artifacts::crate_file(name, version, Some("Example")).expect("builds");
        let body = artifacts::cargo_publish_body(&metadata(name, version), &crate_file);
        self.server
            .request_with_bare_token(
                "PUT",
                &format!("{}/api/v1/crates/new", base()),
                &self.token,
                body,
            )
            .await
    }

    /// The index lines for a crate, parsed.
    async fn index(&self, name: &str) -> Vec<Value> {
        let path = index_path(name);
        let response = self.server.get(&format!("{}/index/{path}", base())).await;
        assert_eq!(
            response.status,
            StatusCode::OK,
            "index fetch failed: {}",
            response.text
        );
        response
            .text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("each index line should be JSON"))
            .collect()
    }
}

/// Mirrors the server's own prefix rule, written out separately so a test would catch the server
/// changing it.
fn index_path(name: &str) -> String {
    let lower = name.to_lowercase();
    match lower.len() {
        1 => format!("1/{lower}"),
        2 => format!("2/{lower}"),
        3 => format!("3/{}/{lower}", &lower[0..1]),
        _ => format!("{}/{}/{lower}", &lower[0..2], &lower[2..4]),
    }
}

#[tokio::test]
async fn a_published_crate_appears_in_the_index() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "a_published_crate_appears_in_the_index"
        ));
        return;
    };

    let response = registry.publish("example", "1.0.0").await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "publish failed: {}",
        response.text
    );

    let lines = registry.index("example").await;
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["name"], "example");
    assert_eq!(lines[0]["vers"], "1.0.0");
    assert_eq!(lines[0]["yanked"], false);
    assert_eq!(lines[0]["v"], 2);

    // `cksum` is what cargo verifies every download against, so it has to be the digest of the
    // bytes the registry actually stored — not merely present.
    let expected =
        artifacts::sha256_hex(&artifacts::crate_file("example", "1.0.0", Some("Example")).unwrap());
    assert_eq!(lines[0]["cksum"], expected);
}

#[tokio::test]
async fn a_published_crate_can_be_downloaded() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database("a_published_crate_can_be_downloaded"));
        return;
    };
    registry.publish("example", "1.0.0").await;

    let response = registry
        .server
        .get(&format!("{}/api/v1/crates/example/1.0.0/download", base()))
        .await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "download failed: {}",
        response.text
    );

    let expected = artifacts::crate_file("example", "1.0.0", Some("Example")).unwrap();
    assert_eq!(response.bytes, expected);
}

#[tokio::test]
async fn the_index_path_is_correct_for_every_name_length() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "the_index_path_is_correct_for_every_name_length"
        ));
        return;
    };

    // One name per bucket of the prefix rule: `1/`, `2/`, `3/{c}/` and `{c1c2}/{c3c4}/`.
    for name in ["a", "ab", "abc", "abcd", "example"] {
        let response = registry.publish(name, "1.0.0").await;
        assert_eq!(
            response.status,
            StatusCode::OK,
            "publishing `{name}` failed: {}",
            response.text
        );
        let lines = registry.index(name).await;
        assert_eq!(lines[0]["name"], name, "wrong crate served for `{name}`");
    }
}

/// A prefix that does not match the name would let one crate be served under another's path, and a
/// cargo client caches by path.
#[tokio::test]
async fn an_index_path_that_does_not_match_the_name_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "an_index_path_that_does_not_match_the_name_is_refused"
        ));
        return;
    };
    registry.publish("example", "1.0.0").await;

    // `example` lives at `ex/am/example`.
    let response = registry
        .server
        .get(&format!("{}/index/aa/bb/example", base()))
        .await;
    assert_eq!(response.status, StatusCode::NOT_FOUND, "{}", response.text);
}

#[tokio::test]
async fn republishing_an_existing_version_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "republishing_an_existing_version_is_refused"
        ));
        return;
    };
    assert_eq!(
        registry.publish("example", "1.0.0").await.status,
        StatusCode::OK
    );

    let second = registry.publish("example", "1.0.0").await;
    assert_eq!(second.status, StatusCode::CONFLICT, "{}", second.text);
    // Cargo prints `errors[].detail`; anything else reaches the user as a bare status.
    assert!(
        second.json()["errors"][0]["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("already exists"),
        "{}",
        second.text
    );

    // The first publish must still be intact.
    let lines = registry.index("example").await;
    assert_eq!(lines.len(), 1);
}

#[tokio::test]
async fn yank_marks_the_version_and_unyank_clears_it() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "yank_marks_the_version_and_unyank_clears_it"
        ));
        return;
    };
    registry.publish("example", "1.0.0").await;

    let yank = registry
        .server
        .request_with_bare_token(
            "DELETE",
            &format!("{}/api/v1/crates/example/1.0.0/yank", base()),
            &registry.token,
            Vec::new(),
        )
        .await;
    assert_eq!(yank.status, StatusCode::OK, "{}", yank.text);
    assert_eq!(registry.index("example").await[0]["yanked"], true);

    // A yanked version is still downloadable — a lockfile that already pins it must keep working.
    let download = registry
        .server
        .get(&format!("{}/api/v1/crates/example/1.0.0/download", base()))
        .await;
    assert_eq!(download.status, StatusCode::OK, "{}", download.text);

    let unyank = registry
        .server
        .request_with_bare_token(
            "PUT",
            &format!("{}/api/v1/crates/example/1.0.0/unyank", base()),
            &registry.token,
            Vec::new(),
        )
        .await;
    assert_eq!(unyank.status, StatusCode::OK, "{}", unyank.text);
    assert_eq!(registry.index("example").await[0]["yanked"], false);
}

#[tokio::test]
async fn config_json_tells_cargo_where_to_find_the_api() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "config_json_tells_cargo_where_to_find_the_api"
        ));
        return;
    };

    let response = registry
        .server
        .get(&format!("{}/index/config.json", base()))
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);

    let config = response.json();
    let expected = format!("http://localhost:6742{}", base());
    assert_eq!(config["api"], expected);
    assert_eq!(config["dl"], format!("{expected}/api/v1/crates"));
    // The registry is public, so cargo should not be told to send a token.
    assert_eq!(config["auth-required"], false);
}

/// A custom domain has to describe *itself* — telling cargo to go back through
/// `/repositories/{storage}/{name}` would work only where both hosts are reachable.
#[tokio::test]
async fn config_json_follows_a_custom_domain() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database("config_json_follows_a_custom_domain"));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, STORAGE, REPOSITORY, "cargo")
        .await;
    let added = server
        .add_hostname(&session, &repository, "crates.example.com")
        .await;
    assert!(added.status.is_success(), "{}", added.text);

    let response = server
        .get_with_host("crates.example.com", "/index/config.json")
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);

    let config = response.json();
    assert_eq!(config["api"], "http://crates.example.com");
    assert_eq!(config["dl"], "http://crates.example.com/api/v1/crates");
}

#[tokio::test]
async fn an_anonymous_publish_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database("an_anonymous_publish_is_refused"));
        return;
    };

    let crate_file = artifacts::crate_file("example", "1.0.0", None).unwrap();
    let body = artifacts::cargo_publish_body(&metadata("example", "1.0.0"), &crate_file);
    let response = registry
        .server
        .request(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("{}/api/v1/crates/new", base()))
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await;
    assert_eq!(
        response.status,
        StatusCode::UNAUTHORIZED,
        "{}",
        response.text
    );

    // Nothing should have been recorded.
    let index = registry
        .server
        .get(&format!("{}/index/ex/am/example", base()))
        .await;
    assert_eq!(index.status, StatusCode::NOT_FOUND);
}

/// The whole point of the scheme-less `Authorization` header change: cargo sends the token bare,
/// and the parser used to reject anything that was not `<scheme> <value>`.
#[tokio::test]
async fn a_bare_authorization_token_is_accepted() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "a_bare_authorization_token_is_accepted"
        ));
        return;
    };

    let response = registry.publish("example", "1.0.0").await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "a bare token should authenticate: {}",
        response.text
    );
}

#[tokio::test]
async fn the_index_answers_conditional_requests() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "the_index_answers_conditional_requests"
        ));
        return;
    };
    registry.publish("example", "1.0.0").await;

    let first = registry
        .server
        .get(&format!("{}/index/ex/am/example", base()))
        .await;
    let etag = first
        .headers
        .get("etag")
        .expect("the index should carry an ETag")
        .to_str()
        .unwrap()
        .to_owned();

    let second = registry
        .server
        .get_with_headers(
            &format!("{}/index/ex/am/example", base()),
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(second.status, StatusCode::NOT_MODIFIED, "{}", second.text);

    // Publishing another version must change the ETag, or a client would never see it.
    registry.publish("example", "1.1.0").await;
    let third = registry
        .server
        .get_with_headers(
            &format!("{}/index/ex/am/example", base()),
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(third.status, StatusCode::OK, "{}", third.text);
    assert_eq!(registry.index("example").await.len(), 2);
}

#[tokio::test]
async fn the_publisher_becomes_the_crates_owner() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "the_publisher_becomes_the_crates_owner"
        ));
        return;
    };
    registry.publish("example", "1.0.0").await;

    let response = registry
        .server
        .get(&format!("{}/api/v1/crates/example/owners", base()))
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);

    let owners = response.json();
    let logins: Vec<&str> = owners["users"]
        .as_array()
        .expect("users should be a list")
        .iter()
        .map(|user| user["login"].as_str().unwrap())
        .collect();
    assert_eq!(logins, vec![common::TEST_USERNAME]);
}

#[tokio::test]
async fn search_finds_a_published_crate() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database("search_finds_a_published_crate"));
        return;
    };
    registry.publish("example", "1.0.0").await;
    registry.publish("unrelated", "2.0.0").await;

    let response = registry
        .server
        .get(&format!("{}/api/v1/crates?q=exam&per_page=10", base()))
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.text);

    let results = response.json();
    let names: Vec<&str> = results["crates"]
        .as_array()
        .expect("crates should be a list")
        .iter()
        .map(|found| found["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["example"]);
    assert_eq!(results["crates"][0]["max_version"], "1.0.0");
}

/// A malformed frame must be refused before anything is allocated for it, and must not leave a
/// half-created project behind.
#[tokio::test]
async fn a_malformed_publish_body_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database("a_malformed_publish_body_is_refused"));
        return;
    };

    // A metadata length of u32::MAX with two bytes behind it.
    let mut body = u32::MAX.to_le_bytes().to_vec();
    body.extend_from_slice(b"{}");

    let response = registry
        .server
        .request_with_bare_token(
            "PUT",
            &format!("{}/api/v1/crates/new", base()),
            &registry.token,
            body,
        )
        .await;
    assert_eq!(
        response.status,
        StatusCode::BAD_REQUEST,
        "{}",
        response.text
    );
}
