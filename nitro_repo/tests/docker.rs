//! Docker registry integration tests.
//!
//! Drives the real router with the request sequence `docker push` and `docker pull` send: the
//! `/v2/` version probe, the bearer challenge and token exchange, `POST`/`PATCH`/`PUT` for each
//! blob, then the manifest. No Docker daemon is involved — the bytes are built here so the tests
//! can assert on things a successful push would never reveal, such as what happens to a blob whose
//! digest is wrong.

mod common;

use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::STANDARD};
use common::{TestResponse, TestServer, skip_without_database};
use nitro_repo::seed::artifacts;

const STORAGE: &str = "local";
const REPOSITORY: &str = "docker";
/// In prefix mode the storage and repository are the first two segments of the image name.
const IMAGE: &str = "local/docker/alpine";

const OCI_MANIFEST_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";

struct Registry {
    server: TestServer,
    token: String,
}

/// The bytes of one image: a layer, a config that references it, and a manifest over both.
struct Image {
    layer: Vec<u8>,
    config: Vec<u8>,
    manifest: Vec<u8>,
}

impl Image {
    fn build(marker: &str) -> Self {
        let layer = artifacts::oci_layer("hello.txt", marker).expect("builds");
        let config = artifacts::oci_config(&layer);
        let manifest = artifacts::oci_manifest(&config, &layer.gzipped);
        Self {
            layer: layer.gzipped,
            config,
            manifest,
        }
    }

    fn digest(&self) -> String {
        artifacts::oci_digest(&self.manifest)
    }
}

impl Registry {
    async fn start() -> Option<Self> {
        let server = TestServer::start().await?;
        server.install().await;
        let session = server.sign_in().await;
        let repository = server
            .create_repository(&session, STORAGE, REPOSITORY, "docker")
            .await;
        let token = server
            .create_repository_token(&session, &repository, &["Read", "Write", "Edit"])
            .await;
        Some(Self { server, token })
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }

    /// Uploads one blob the way a client does: open a session, stream it, commit it by digest.
    async fn push_blob(&self, image: &str, bytes: &[u8]) -> TestResponse {
        let digest = artifacts::oci_digest(bytes);

        let started = self
            .server
            .send(
                "POST",
                &format!("/v2/{image}/blobs/uploads/"),
                &[("authorization", &self.auth())],
                Vec::new(),
            )
            .await;
        assert_eq!(
            started.status,
            StatusCode::ACCEPTED,
            "opening an upload failed: {}",
            started.text
        );
        let location = started
            .headers
            .get("location")
            .expect("an opened upload should carry a Location")
            .to_str()
            .unwrap()
            .to_owned();

        let patched = self
            .server
            .send(
                "PATCH",
                &location,
                &[
                    ("authorization", &self.auth()),
                    (
                        "content-range",
                        &format!("0-{}", bytes.len().saturating_sub(1)),
                    ),
                ],
                bytes.to_vec(),
            )
            .await;
        assert_eq!(
            patched.status,
            StatusCode::ACCEPTED,
            "streaming a blob failed: {}",
            patched.text
        );

        self.server
            .send(
                "PUT",
                &format!("{location}?digest={digest}"),
                &[("authorization", &self.auth())],
                Vec::new(),
            )
            .await
    }

    async fn push_manifest(&self, image: &str, reference: &str, manifest: &[u8]) -> TestResponse {
        self.server
            .send(
                "PUT",
                &format!("/v2/{image}/manifests/{reference}"),
                &[
                    ("authorization", &self.auth()),
                    ("content-type", OCI_MANIFEST_TYPE),
                ],
                manifest.to_vec(),
            )
            .await
    }

    /// The whole of `docker push`.
    async fn push(&self, image: &str, tag: &str, built: &Image) -> TestResponse {
        assert_eq!(
            self.push_blob(image, &built.layer).await.status,
            StatusCode::CREATED
        );
        assert_eq!(
            self.push_blob(image, &built.config).await.status,
            StatusCode::CREATED
        );
        self.push_manifest(image, tag, &built.manifest).await
    }

    async fn get(&self, path: &str) -> TestResponse {
        self.server
            .send("GET", path, &[("authorization", &self.auth())], Vec::new())
            .await
    }
}

#[tokio::test]
async fn an_image_round_trips() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database("an_image_round_trips"));
        return;
    };
    let built = Image::build("round trip");

    let pushed = registry.push(IMAGE, "1.0", &built).await;
    assert_eq!(
        pushed.status,
        StatusCode::CREATED,
        "pushing the manifest failed: {}",
        pushed.text
    );
    assert_eq!(
        pushed.headers.get("docker-content-digest").unwrap(),
        built.digest().as_str()
    );

    // By tag.
    let by_tag = registry.get(&format!("/v2/{IMAGE}/manifests/1.0")).await;
    assert_eq!(by_tag.status, StatusCode::OK, "{}", by_tag.text);
    assert_eq!(by_tag.bytes, built.manifest);
    assert_eq!(
        by_tag.headers.get("content-type").unwrap(),
        OCI_MANIFEST_TYPE
    );
    assert_eq!(
        by_tag.headers.get("docker-content-digest").unwrap(),
        built.digest().as_str()
    );

    // By digest — the same bytes, which is what makes the digest meaningful.
    let by_digest = registry
        .get(&format!("/v2/{IMAGE}/manifests/{}", built.digest()))
        .await;
    assert_eq!(by_digest.status, StatusCode::OK, "{}", by_digest.text);
    assert_eq!(by_digest.bytes, built.manifest);

    // And the layer comes back byte-for-byte.
    let layer = registry
        .get(&format!(
            "/v2/{IMAGE}/blobs/{}",
            artifacts::oci_digest(&built.layer)
        ))
        .await;
    assert_eq!(layer.status, StatusCode::OK, "{}", layer.text);
    assert_eq!(layer.bytes, built.layer);
}

/// A push re-uploads every layer unless `HEAD` can tell it what is already here, so this is the
/// difference between a fast push and a slow one.
#[tokio::test]
async fn head_reports_what_is_already_present() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "head_reports_what_is_already_present"
        ));
        return;
    };
    let built = Image::build("head");
    let layer_digest = artifacts::oci_digest(&built.layer);

    let missing = registry
        .server
        .send(
            "HEAD",
            &format!("/v2/{IMAGE}/blobs/{layer_digest}"),
            &[("authorization", &registry.auth())],
            Vec::new(),
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    registry.push(IMAGE, "1.0", &built).await;

    let present = registry
        .server
        .send(
            "HEAD",
            &format!("/v2/{IMAGE}/blobs/{layer_digest}"),
            &[("authorization", &registry.auth())],
            Vec::new(),
        )
        .await;
    assert_eq!(present.status, StatusCode::OK, "{}", present.text);
    // Same headers as the GET, no body.
    assert_eq!(
        present.headers.get("content-length").unwrap(),
        built.layer.len().to_string().as_str()
    );
    assert!(present.bytes.is_empty());

    let manifest_head = registry
        .server
        .send(
            "HEAD",
            &format!("/v2/{IMAGE}/manifests/1.0"),
            &[("authorization", &registry.auth())],
            Vec::new(),
        )
        .await;
    assert_eq!(manifest_head.status, StatusCode::OK);
    assert_eq!(
        manifest_head.headers.get("docker-content-digest").unwrap(),
        built.digest().as_str()
    );
    assert!(manifest_head.bytes.is_empty());
}

#[tokio::test]
async fn a_blob_whose_digest_does_not_match_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "a_blob_whose_digest_does_not_match_is_refused"
        ));
        return;
    };

    let started = registry
        .server
        .send(
            "POST",
            &format!("/v2/{IMAGE}/blobs/uploads/"),
            &[("authorization", &registry.auth())],
            Vec::new(),
        )
        .await;
    let location = started
        .headers
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    registry
        .server
        .send(
            "PATCH",
            &location,
            &[("authorization", &registry.auth())],
            b"the real content".to_vec(),
        )
        .await;

    // Committed under the digest of something else entirely.
    let wrong = artifacts::oci_digest(b"not what was uploaded");
    let committed = registry
        .server
        .send(
            "PUT",
            &format!("{location}?digest={wrong}"),
            &[("authorization", &registry.auth())],
            Vec::new(),
        )
        .await;
    assert_eq!(
        committed.status,
        StatusCode::BAD_REQUEST,
        "{}",
        committed.text
    );
    assert_eq!(committed.json()["errors"][0]["code"], "DIGEST_INVALID");

    // Nothing must have reached storage under either digest.
    for digest in [wrong, artifacts::oci_digest(b"the real content")] {
        let fetched = registry.get(&format!("/v2/{IMAGE}/blobs/{digest}")).await;
        assert_eq!(fetched.status, StatusCode::NOT_FOUND, "{digest}");
    }
}

/// A manifest accepted before its layers are here produces an image that pushes fine and fails
/// every pull, with nothing to say which push was at fault.
#[tokio::test]
async fn a_manifest_referencing_a_missing_blob_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "a_manifest_referencing_a_missing_blob_is_refused"
        ));
        return;
    };
    let built = Image::build("missing blob");

    // The config is pushed but the layer is not.
    assert_eq!(
        registry.push_blob(IMAGE, &built.config).await.status,
        StatusCode::CREATED
    );

    let pushed = registry.push_manifest(IMAGE, "1.0", &built.manifest).await;
    assert_eq!(pushed.status, StatusCode::BAD_REQUEST, "{}", pushed.text);
    assert_eq!(pushed.json()["errors"][0]["code"], "MANIFEST_BLOB_UNKNOWN");

    let fetched = registry.get(&format!("/v2/{IMAGE}/manifests/1.0")).await;
    assert_eq!(fetched.status, StatusCode::NOT_FOUND);
}

/// A manifest pushed by digest must actually hash to that digest, or the registry would serve
/// content under an identity that is not its own.
#[tokio::test]
async fn a_manifest_pushed_under_the_wrong_digest_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "a_manifest_pushed_under_the_wrong_digest_is_refused"
        ));
        return;
    };
    let built = Image::build("wrong digest");
    registry.push_blob(IMAGE, &built.layer).await;
    registry.push_blob(IMAGE, &built.config).await;

    let wrong = artifacts::oci_digest(b"a different manifest");
    let pushed = registry.push_manifest(IMAGE, &wrong, &built.manifest).await;
    assert_eq!(pushed.status, StatusCode::BAD_REQUEST, "{}", pushed.text);
    assert_eq!(pushed.json()["errors"][0]["code"], "DIGEST_INVALID");
}

#[tokio::test]
async fn tags_list_and_catalog_reflect_what_was_pushed() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "tags_list_and_catalog_reflect_what_was_pushed"
        ));
        return;
    };

    for tag in ["1.0", "1.1", "latest"] {
        let built = Image::build(tag);
        assert_eq!(
            registry.push(IMAGE, tag, &built).await.status,
            StatusCode::CREATED
        );
    }
    let other = Image::build("other");
    registry.push("local/docker/busybox", "1.0", &other).await;

    let tags = registry.get(&format!("/v2/{IMAGE}/tags/list")).await;
    assert_eq!(tags.status, StatusCode::OK, "{}", tags.text);
    let body = tags.json();
    assert_eq!(body["name"], IMAGE);
    // Lexically ordered, which is what the spec says and what makes `?last=` paging meaningful.
    assert_eq!(
        body["tags"].as_array().unwrap(),
        &vec!["1.0", "1.1", "latest"]
    );

    let catalog = registry.get("/v2/local/docker/_catalog").await;
    assert_eq!(catalog.status, StatusCode::OK, "{}", catalog.text);
    let repositories = catalog.json();
    let names: Vec<&str> = repositories["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["local/docker/alpine", "local/docker/busybox"]);
}

/// `:latest` moves with every push. A repository that refused to re-tag would be unusable.
#[tokio::test]
async fn a_tag_can_be_moved_to_a_new_manifest() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "a_tag_can_be_moved_to_a_new_manifest"
        ));
        return;
    };

    let first = Image::build("first");
    assert_eq!(
        registry.push(IMAGE, "latest", &first).await.status,
        StatusCode::CREATED
    );

    let second = Image::build("second");
    assert_ne!(first.digest(), second.digest());
    assert_eq!(
        registry.push(IMAGE, "latest", &second).await.status,
        StatusCode::CREATED
    );

    let fetched = registry.get(&format!("/v2/{IMAGE}/manifests/latest")).await;
    assert_eq!(fetched.bytes, second.manifest);

    // The old manifest is still reachable by digest — only the tag moved.
    let old = registry
        .get(&format!("/v2/{IMAGE}/manifests/{}", first.digest()))
        .await;
    assert_eq!(old.status, StatusCode::OK, "{}", old.text);
    assert_eq!(old.bytes, first.manifest);

    // And there is still exactly one tag.
    let tags = registry.get(&format!("/v2/{IMAGE}/tags/list")).await;
    assert_eq!(tags.json()["tags"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn an_anonymous_push_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database("an_anonymous_push_is_refused"));
        return;
    };

    let started = registry
        .server
        .send(
            "POST",
            &format!("/v2/{IMAGE}/blobs/uploads/"),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(started.status, StatusCode::UNAUTHORIZED, "{}", started.text);
    // A client that is not challenged never tries to authenticate.
    let challenge = started
        .headers
        .get("www-authenticate")
        .expect("an anonymous push should be challenged")
        .to_str()
        .unwrap();
    assert!(challenge.starts_with("Bearer realm="), "{challenge}");
    assert!(
        challenge.contains(&format!("scope=\"repository:{IMAGE}:pull,push\"")),
        "{challenge}"
    );
}

/// The `/v2/` probe and the realm exchange, which is the whole of `docker login`.
#[tokio::test]
async fn the_token_endpoint_mints_a_usable_credential() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "the_token_endpoint_mints_a_usable_credential"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    server
        .create_repository(&session, STORAGE, REPOSITORY, "docker")
        .await;

    // A public registry answers the probe without credentials.
    let probe = server.get("/v2/").await;
    assert_eq!(probe.status, StatusCode::OK, "{}", probe.text);
    assert_eq!(
        probe
            .headers
            .get("docker-distribution-api-version")
            .unwrap(),
        "registry/2.0"
    );

    let credentials = STANDARD.encode(format!(
        "{}:{}",
        common::TEST_USERNAME,
        common::TEST_PASSWORD
    ));
    let minted = server
        .send(
            "GET",
            &format!("/api/docker/token?service=nitro&scope=repository:{IMAGE}:pull,push"),
            &[("authorization", &format!("Basic {credentials}"))],
            Vec::new(),
        )
        .await;
    assert_eq!(minted.status, StatusCode::OK, "{}", minted.text);

    let body = minted.json();
    let token = body["token"].as_str().expect("a token").to_owned();
    // Both names are sent because different clients read different ones.
    assert_eq!(body["access_token"].as_str(), Some(token.as_str()));
    assert!(body["expires_in"].as_i64().unwrap() > 0);

    // And the token actually authenticates a push.
    let registry = Registry { server, token };
    let built = Image::build("via token");
    assert_eq!(
        registry.push(IMAGE, "1.0", &built).await.status,
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn bad_credentials_do_not_mint_a_token() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database("bad_credentials_do_not_mint_a_token"));
        return;
    };
    server.install().await;

    let credentials = STANDARD.encode(format!("{}:wrong-password", common::TEST_USERNAME));
    let refused = server
        .send(
            "GET",
            "/api/docker/token?scope=repository:local/docker/alpine:pull",
            &[("authorization", &format!("Basic {credentials}"))],
            Vec::new(),
        )
        .await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED, "{}", refused.text);
}

/// Hostname mode: the image name carries no storage/repository prefix, because the host already
/// says which repository this is.
#[tokio::test]
async fn an_image_round_trips_over_a_custom_domain() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "an_image_round_trips_over_a_custom_domain"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, STORAGE, REPOSITORY, "docker")
        .await;
    let added = server
        .add_hostname(&session, &repository, "docker.example.com")
        .await;
    assert!(added.status.is_success(), "{}", added.text);
    let token = server
        .create_repository_token(&session, &repository, &["Read", "Write"])
        .await;
    let auth = format!("Bearer {token}");

    let built = Image::build("custom domain");
    let host = [("host", "docker.example.com")];

    // Blob, then manifest, all under the bare image name.
    for blob in [&built.layer, &built.config] {
        let started = server
            .send(
                "POST",
                "/v2/alpine/blobs/uploads/",
                &[("authorization", &auth), host[0]],
                Vec::new(),
            )
            .await;
        assert_eq!(started.status, StatusCode::ACCEPTED, "{}", started.text);
        let location = started
            .headers
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        server
            .send(
                "PATCH",
                &location,
                &[("authorization", &auth), host[0]],
                blob.to_vec(),
            )
            .await;
        let committed = server
            .send(
                "PUT",
                &format!("{location}?digest={}", artifacts::oci_digest(blob)),
                &[("authorization", &auth), host[0]],
                Vec::new(),
            )
            .await;
        assert_eq!(committed.status, StatusCode::CREATED, "{}", committed.text);
    }

    let pushed = server
        .send(
            "PUT",
            "/v2/alpine/manifests/1.0",
            &[
                ("authorization", &auth),
                ("content-type", OCI_MANIFEST_TYPE),
                host[0],
            ],
            built.manifest.clone(),
        )
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text);

    let fetched = server
        .send(
            "GET",
            "/v2/alpine/manifests/1.0",
            &[("authorization", &auth), host[0]],
            Vec::new(),
        )
        .await;
    assert_eq!(fetched.status, StatusCode::OK, "{}", fetched.text);
    assert_eq!(fetched.bytes, built.manifest);
}

/// `/v2` is mounted at the host root, so it must not swallow a Maven artifact whose path happens to
/// begin with `v2/` on a custom domain.
#[tokio::test]
async fn a_v2_path_on_a_maven_domain_still_serves_the_artifact() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_v2_path_on_a_maven_domain_still_serves_the_artifact"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, STORAGE, "releases", "maven")
        .await;
    let added = server
        .add_hostname(&session, &repository, "maven.example.com")
        .await;
    assert!(added.status.is_success(), "{}", added.text);

    let deployed = server
        .put_with_basic_and_host(
            "maven.example.com",
            "/v2/dev/kingtux/tms/1.0.0/tms-1.0.0.jar",
            b"not really a jar".to_vec(),
            "application/java-archive",
        )
        .await;
    assert!(
        deployed.status.is_success(),
        "deploy under a v2/ path failed: {} {}",
        deployed.status,
        deployed.text
    );

    let fetched = server
        .get_with_host(
            "maven.example.com",
            "/v2/dev/kingtux/tms/1.0.0/tms-1.0.0.jar",
        )
        .await;
    assert_eq!(fetched.status, StatusCode::OK, "{}", fetched.text);
    assert_eq!(fetched.bytes, b"not really a jar");
}

#[tokio::test]
async fn deleting_a_tag_leaves_the_manifest_reachable_by_digest() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "deleting_a_tag_leaves_the_manifest_reachable_by_digest"
        ));
        return;
    };
    let built = Image::build("delete");
    registry.push(IMAGE, "1.0", &built).await;

    let deleted = registry
        .server
        .send(
            "DELETE",
            &format!("/v2/{IMAGE}/manifests/1.0"),
            &[("authorization", &registry.auth())],
            Vec::new(),
        )
        .await;
    assert_eq!(deleted.status, StatusCode::ACCEPTED, "{}", deleted.text);

    assert_eq!(
        registry
            .get(&format!("/v2/{IMAGE}/manifests/1.0"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    // The manifest itself is untouched — another tag or a pinned digest may still want it.
    assert_eq!(
        registry
            .get(&format!("/v2/{IMAGE}/manifests/{}", built.digest()))
            .await
            .status,
        StatusCode::OK
    );
}

/// Nothing reference-counts layers, so deleting one would corrupt every other manifest sharing it.
#[tokio::test]
async fn deleting_a_blob_is_refused() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database("deleting_a_blob_is_refused"));
        return;
    };
    let built = Image::build("blob delete");
    registry.push(IMAGE, "1.0", &built).await;

    let refused = registry
        .server
        .send(
            "DELETE",
            &format!("/v2/{IMAGE}/blobs/{}", artifacts::oci_digest(&built.layer)),
            &[("authorization", &registry.auth())],
            Vec::new(),
        )
        .await;
    assert_eq!(
        refused.status,
        StatusCode::METHOD_NOT_ALLOWED,
        "{}",
        refused.text
    );
    assert_eq!(refused.json()["errors"][0]["code"], "UNSUPPORTED");
}

#[tokio::test]
async fn an_unresolvable_prefix_says_what_was_expected() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "an_unresolvable_prefix_says_what_was_expected"
        ));
        return;
    };

    // One segment: there is no `{storage}/{repository}` in it at all.
    let single = registry.get("/v2/alpine/manifests/latest").await;
    assert_eq!(single.status, StatusCode::NOT_FOUND, "{}", single.text);
    assert_eq!(single.json()["errors"][0]["code"], "NAME_UNKNOWN");

    // Two segments that name nothing.
    let missing = registry.get("/v2/nope/nothing/manifests/latest").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND, "{}", missing.text);
    assert!(
        missing.json()["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("nope/nothing"),
        "{}",
        missing.text
    );
}

/// A private registry must challenge rather than refuse outright, or the client never sends
/// credentials.
#[tokio::test]
async fn a_private_registry_challenges_an_anonymous_pull() {
    let Some(server) = TestServer::start().await else {
        assert!(skip_without_database(
            "a_private_registry_challenges_an_anonymous_pull"
        ));
        return;
    };
    server.install().await;
    let session = server.sign_in().await;
    let repository = server
        .create_repository(&session, STORAGE, REPOSITORY, "docker")
        .await;
    let updated = server
        .put_json_as(
            &format!("/api/repository/{repository}"),
            serde_json::json!({ "visibility": "Private", "active": true }),
            &session,
        )
        .await;
    assert!(updated.status.is_success(), "{}", updated.text);

    let refused = server.get(&format!("/v2/{IMAGE}/manifests/latest")).await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED, "{}", refused.text);
    assert!(
        refused
            .headers
            .get("www-authenticate")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("Bearer realm="),
    );
}

#[tokio::test]
async fn an_index_can_be_pushed_once_its_children_are_present() {
    let Some(registry) = Registry::start().await else {
        assert!(skip_without_database(
            "an_index_can_be_pushed_once_its_children_are_present"
        ));
        return;
    };

    let amd64 = Image::build("amd64");
    let arm64 = Image::build("arm64");

    let index = serde_json::json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.index.v1+json",
        "manifests": [
            {
                "mediaType": OCI_MANIFEST_TYPE,
                "digest": amd64.digest(),
                "size": amd64.manifest.len(),
                "platform": { "os": "linux", "architecture": "amd64" },
            },
            {
                "mediaType": OCI_MANIFEST_TYPE,
                "digest": arm64.digest(),
                "size": arm64.manifest.len(),
                "platform": { "os": "linux", "architecture": "arm64" },
            },
        ],
    });
    let index_bytes = serde_json::to_vec(&index).unwrap();

    // Before the children exist, the index must be refused.
    let early = registry
        .server
        .send(
            "PUT",
            &format!("/v2/{IMAGE}/manifests/multi"),
            &[
                ("authorization", &registry.auth()),
                ("content-type", "application/vnd.oci.image.index.v1+json"),
            ],
            index_bytes.clone(),
        )
        .await;
    assert_eq!(early.status, StatusCode::BAD_REQUEST, "{}", early.text);
    assert_eq!(early.json()["errors"][0]["code"], "MANIFEST_BLOB_UNKNOWN");

    // Push each platform by digest, then the index.
    for built in [&amd64, &arm64] {
        registry.push_blob(IMAGE, &built.layer).await;
        registry.push_blob(IMAGE, &built.config).await;
        let pushed = registry
            .push_manifest(IMAGE, &built.digest(), &built.manifest)
            .await;
        assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text);
    }

    let pushed = registry
        .server
        .send(
            "PUT",
            &format!("/v2/{IMAGE}/manifests/multi"),
            &[
                ("authorization", &registry.auth()),
                ("content-type", "application/vnd.oci.image.index.v1+json"),
            ],
            index_bytes.clone(),
        )
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text);

    let fetched = registry.get(&format!("/v2/{IMAGE}/manifests/multi")).await;
    assert_eq!(fetched.bytes, index_bytes);
    assert_eq!(
        fetched.headers.get("content-type").unwrap(),
        "application/vnd.oci.image.index.v1+json"
    );

    // The per-platform manifests are reachable by digest but are not tags.
    let tags = registry.get(&format!("/v2/{IMAGE}/tags/list")).await;
    assert_eq!(tags.json()["tags"].as_array().unwrap(), &vec!["multi"]);
}
