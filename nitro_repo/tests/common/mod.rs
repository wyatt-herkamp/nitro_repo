//! The integration test harness. (#502)
//!
//! Each test gets its own Postgres database and its own storage directory, and drives the *real*
//! router — the one `web::start` serves, with the real middleware stack — through
//! `tower::ServiceExt::oneshot`. Nothing binds a port, so tests run in parallel without fighting
//! over one, and there is no server process to start, wait for, or leak.
//!
//! Requires a Postgres reachable at `NITRO_TEST_DATABASE_URL` (or the `DATABASE_URL` in
//! `nr_tests.env`). Without one, [`TestServer::start`] returns `None` and the test skips, unless
//! `NITRO_TESTS_REQUIRE_DB=1` is set — CI sets it, so a skip there is a failure.

#![allow(dead_code)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use nitro_repo::app::{
    NitroRepo,
    config::{Mode, SecuritySettings, SiteSetting},
    web::build_app,
};
use nr_core::database::DatabaseConfig;
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection};
use tower::ServiceExt;

pub static TEST_USERNAME: &str = "test_admin";
pub static TEST_PASSWORD: &str = "TestPassword1";

/// Names each test's database uniquely, so a parallel run does not share state.
static DATABASE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct TestServer {
    pub router: Router,
    pub site: NitroRepo,
    /// Held so the directory outlives the test; dropping it removes the storage and session files.
    _directory: tempfile::TempDir,
    database_url: String,
    database_name: String,
}

/// Where the test Postgres lives, minus the database name.
fn postgres_root() -> Option<String> {
    if let Ok(url) = std::env::var("NITRO_TEST_DATABASE_URL") {
        return Some(url);
    }
    // Falls back to the same file `TestCore` reads, so one setup serves both.
    let contents = std::fs::read_to_string(find_upwards("nr_tests.env")?).ok()?;
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("DATABASE_URL=") {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn find_upwards(name: &str) -> Option<PathBuf> {
    let mut directory = std::env::current_dir().ok()?;
    loop {
        let candidate = directory.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
        directory = directory.parent()?.to_path_buf();
    }
}

/// Skips, or fails, depending on whether the environment promised a database.
pub fn skip_without_database(what: &str) -> bool {
    if std::env::var("NITRO_TESTS_REQUIRE_DB").as_deref() == Ok("1") {
        panic!(
            "{what} needs a database and NITRO_TESTS_REQUIRE_DB=1 was set. \
             Start one with `just services-up --profile postgres`."
        );
    }
    eprintln!(
        "skipping {what}: no test database. Set NITRO_TEST_DATABASE_URL or run `just services-up`."
    );
    true
}

impl TestServer {
    /// Builds an instance against a fresh database. `None` when there is no Postgres to use.
    pub async fn start() -> Option<Self> {
        let root = postgres_root()?;

        // `postgres://user:pass@host/some_db` → the same, with a unique database name. Each test
        // gets its own so migrations, seeded rows and unique constraints cannot collide.
        let (prefix, _) = root.rsplit_once('/')?;
        let database_name = format!(
            "nr_it_{}_{}",
            std::process::id(),
            DATABASE_COUNTER.fetch_add(1, Ordering::SeqCst)
        );
        let admin_url = format!("{prefix}/postgres");
        let database_url = format!("{prefix}/{database_name}");

        let mut admin = PgConnection::connect(&admin_url).await.ok()?;
        admin
            .execute(AssertSqlSafe(format!(
                r#"DROP DATABASE IF EXISTS "{database_name}""#
            )))
            .await
            .ok()?;
        admin
            .execute(AssertSqlSafe(format!(
                r#"CREATE DATABASE "{database_name}""#
            )))
            .await
            .ok()?;
        drop(admin);

        let directory = tempfile::tempdir().ok()?;

        let site = NitroRepo::new(
            Mode::Debug,
            SiteSetting {
                app_url: Some("http://localhost:6742/".to_owned()),
                name: "Integration tests".to_owned(),
                description: "Integration tests".to_owned(),
                is_https: false,
                #[cfg(feature = "frontend")]
                frontend_path: None,
            },
            SecuritySettings::default(),
            nitro_repo::app::authentication::session::SessionManagerConfig {
                database_location: directory.path().join("sessions.redb"),
                ..Default::default()
            },
            nitro_repo::repository::staging::StagingConfig {
                staging_dir: directory.path().join("staging"),
                ..Default::default()
            },
            // No email service: nothing under test sends one, and configuring a transport would
            // make these tests depend on a mail server being reachable.
            None,
            parse_database_config(&database_url)?,
            Some(directory.path().join("storage")),
        )
        .await
        .ok()?;

        let router = build_app(site.clone(), false, DefaultBodyLimit::max(64 * 1024 * 1024));

        Some(Self {
            router,
            site,
            _directory: directory,
            database_url,
            database_name,
        })
    }

    /// Creates the first admin, which is what `POST /api/install` does.
    pub async fn install(&self) {
        let response = self
            .post_json(
                "/api/install",
                json!({
                    "user": {
                        "username": TEST_USERNAME,
                        "email": "test@example.com",
                        "name": "Test Admin",
                        "password": TEST_PASSWORD,
                    }
                }),
            )
            .await;
        assert_eq!(
            response.status,
            StatusCode::NO_CONTENT,
            "install failed: {}",
            response.text
        );
    }

    /// Signs in and returns the session id, which the API accepts as `Authorization: Session <id>`.
    pub async fn sign_in(&self) -> String {
        let response = self
            .post_json(
                "/api/user/login",
                json!({ "email_or_username": TEST_USERNAME, "password": TEST_PASSWORD }),
            )
            .await;
        assert_eq!(
            response.status,
            StatusCode::OK,
            "login failed: {}",
            response.text
        );

        response.json()["session"]["session_id"]
            .as_str()
            .expect("the login response should carry a session id")
            .to_owned()
    }

    /// Sends a request through the real router.
    ///
    /// A `user-agent` is added when the caller did not set one: the login route records it against
    /// the session and rejects a request without it, and every real client sends one.
    pub async fn request(&self, mut request: Request<Body>) -> TestResponse {
        if !request.headers().contains_key(header::USER_AGENT) {
            request.headers_mut().insert(
                header::USER_AGENT,
                axum::http::HeaderValue::from_static("nitro-repo-integration-tests"),
            );
        }

        // In production this extension is added by `into_make_service_with_connect_info`, from the
        // accepted TCP connection. There is no connection here, and the routes that record a
        // client address — login, password reset — extract it unconditionally.
        request
            .extensions_mut()
            .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                [127, 0, 0, 1],
                47_000,
            ))));

        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("the router should not fail to respond");

        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("the body should be readable")
            .to_bytes();

        TestResponse {
            status,
            headers,
            text: String::from_utf8_lossy(&bytes).into_owned(),
            bytes: bytes.to_vec(),
        }
    }

    pub async fn get(&self, path: &str) -> TestResponse {
        self.request(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
    }

    pub async fn get_as(&self, path: &str, session: &str) -> TestResponse {
        self.request(
            Request::builder()
                .uri(path)
                .header(header::AUTHORIZATION, format!("Session {session}"))
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
    }

    pub async fn post_json(&self, path: &str, body: Value) -> TestResponse {
        self.request(
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("valid request"),
        )
        .await
    }

    pub async fn post_json_as(&self, path: &str, body: Value, session: &str) -> TestResponse {
        self.request(
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Session {session}"))
                .body(Body::from(body.to_string()))
                .expect("valid request"),
        )
        .await
    }

    pub async fn put_json_as(&self, path: &str, body: Value, session: &str) -> TestResponse {
        self.request(
            Request::builder()
                .method("PUT")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Session {session}"))
                .body(Body::from(body.to_string()))
                .expect("valid request"),
        )
        .await
    }

    /// A `PUT` with basic auth — what `mvn deploy` and `npm publish` actually send.
    pub async fn put_with_basic(
        &self,
        path: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> TestResponse {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let credentials = STANDARD.encode(format!("{TEST_USERNAME}:{TEST_PASSWORD}"));

        self.request(
            Request::builder()
                .method("PUT")
                .uri(path)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::AUTHORIZATION, format!("Basic {credentials}"))
                .body(Body::from(body))
                .expect("valid request"),
        )
        .await
    }

    /// Mints an auth token scoped to one repository, and returns its secret.
    ///
    /// This is how cargo and Docker authenticate — neither ever sees the user's password — so a
    /// test that wants to behave like the real client has to go and get one the same way.
    pub async fn create_repository_token(
        &self,
        session: &str,
        repository_id: &str,
        actions: &[&str],
    ) -> String {
        let response = self
            .post_json_as(
                "/api/user/token/create",
                json!({
                    "name": "integration test",
                    "repository_scopes": [
                        { "repository_id": repository_id, "scopes": actions }
                    ],
                }),
                session,
            )
            .await;
        assert!(
            response.status.is_success(),
            "creating a token failed: {} {}",
            response.status,
            response.text
        );
        response.json()["token"]
            .as_str()
            .expect("the response should carry a token")
            .to_owned()
    }

    /// A request with an arbitrary method, body, headers and (optionally) a `Host`.
    ///
    /// Docker pushes need all four at once — `PATCH` with a `Content-Range`, `PUT` with a
    /// `Content-Type`, both bearing a token — which none of the narrower helpers above cover.
    pub async fn send(
        &self,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> TestResponse {
        let mut builder = Request::builder().method(method).uri(path);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        self.request(builder.body(Body::from(body)).expect("valid request"))
            .await
    }

    /// A request carrying `Authorization: <token>` with no scheme — the way cargo sends it.
    pub async fn request_with_bare_token(
        &self,
        method: &str,
        path: &str,
        token: &str,
        body: Vec<u8>,
    ) -> TestResponse {
        self.request(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::AUTHORIZATION, token)
                .body(Body::from(body))
                .expect("valid request"),
        )
        .await
    }

    /// A `GET` addressed to a specific host.
    ///
    /// `Request::builder().uri("/x")` produces a URI with no authority, so `Host` is the only
    /// signal the router has — which is exactly the path host routing takes in production.
    pub async fn get_with_host(&self, host: &str, path: &str) -> TestResponse {
        self.request(
            Request::builder()
                .uri(path)
                .header(header::HOST, host)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
    }

    pub async fn get_with_headers(&self, path: &str, headers: &[(&str, &str)]) -> TestResponse {
        let mut builder = Request::builder().uri(path);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        self.request(builder.body(Body::empty()).expect("valid request"))
            .await
    }

    /// A `PUT` with basic auth, addressed to a specific host — a domain-routed `mvn deploy`.
    pub async fn put_with_basic_and_host(
        &self,
        host: &str,
        path: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> TestResponse {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let credentials = STANDARD.encode(format!("{TEST_USERNAME}:{TEST_PASSWORD}"));

        self.request(
            Request::builder()
                .method("PUT")
                .uri(path)
                .header(header::HOST, host)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::AUTHORIZATION, format!("Basic {credentials}"))
                .body(Body::from(body))
                .expect("valid request"),
        )
        .await
    }

    /// Registers a custom domain through the API.
    ///
    /// Always through the API, never with a direct `INSERT`: the routing index is in memory, and a
    /// row written behind its back would leave the request falling through to the frontend, which
    /// reads as a routing bug rather than as the test's own mistake.
    pub async fn add_hostname(
        &self,
        session: &str,
        repository_id: &str,
        hostname: &str,
    ) -> TestResponse {
        self.post_json_as(
            &format!("/api/repository/{repository_id}/hostnames"),
            json!({ "hostname": hostname }),
            session,
        )
        .await
    }

    pub async fn delete_as(&self, path: &str, session: &str) -> TestResponse {
        self.request(
            Request::builder()
                .method("DELETE")
                .uri(path)
                .header(header::AUTHORIZATION, format!("Session {session}"))
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
    }

    /// Creates a storage and a repository, and returns the repository's id.
    pub async fn create_repository(
        &self,
        session: &str,
        storage_name: &str,
        repository_name: &str,
        repository_type: &str,
    ) -> String {
        let storages = self.get_as("/api/storage/list", session).await;
        let existing = storages.json().as_array().and_then(|list| {
            list.iter()
                .find(|value| value["name"] == storage_name)
                .and_then(|value| value["id"].as_str().map(str::to_owned))
        });

        let storage_id = match existing {
            Some(id) => id,
            None => {
                let response = self
                    .post_json_as(
                        "/api/storage/new/Local",
                        json!({
                            "name": storage_name,
                            "config": {
                                "type": "Local",
                                "settings": { "path": self._directory.path().join("storage") },
                            },
                        }),
                        session,
                    )
                    .await;
                assert!(
                    response.status.is_success(),
                    "creating storage failed: {} {}",
                    response.status,
                    response.text
                );
                response.json()["id"].as_str().unwrap().to_owned()
            }
        };

        let response = self
            .post_json_as(
                &format!("/api/repository/new/{repository_type}"),
                json!({
                    "name": repository_name,
                    "storage": storage_id,
                    "configs": { repository_type: { "type": "Hosted" } },
                }),
                session,
            )
            .await;
        assert!(
            response.status.is_success(),
            "creating repository failed: {} {}",
            response.status,
            response.text
        );

        response.json()["id"].as_str().unwrap().to_owned()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Dropped databases keep a parallel run from accumulating dozens of them. Best effort: a
        // failure here should not mask whatever the test was actually asserting.
        let Some((prefix, _)) = self.database_url.rsplit_once('/') else {
            return;
        };
        let admin_url = format!("{prefix}/postgres");
        let name = self.database_name.clone();

        let _ = std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(async move {
                if let Ok(mut admin) = PgConnection::connect(&admin_url).await {
                    let _ = admin
                        .execute(AssertSqlSafe(format!(
                            r#"DROP DATABASE IF EXISTS "{name}" WITH (FORCE)"#
                        )))
                        .await;
                }
            });
        })
        .join();
    }
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: axum::http::HeaderMap,
    pub text: String,
    pub bytes: Vec<u8>,
}

impl TestResponse {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.text)
            .unwrap_or_else(|error| panic!("expected JSON, got `{}` ({error})", self.text))
    }
}

fn parse_database_config(url: &str) -> Option<DatabaseConfig> {
    // `postgres://user:password@host:port/database`
    let rest = url.strip_prefix("postgres://")?;
    let (credentials, rest) = rest.split_once('@')?;
    let (user, password) = credentials.split_once(':')?;
    let (host_port, database) = rest.split_once('/')?;
    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) => (host, port.parse().ok()),
        None => (host_port, None),
    };

    Some(DatabaseConfig {
        user: user.to_owned(),
        password: password.to_owned(),
        host: host.to_owned(),
        port,
        database: database.to_owned(),
    })
}
