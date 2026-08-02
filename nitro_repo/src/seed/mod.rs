//! Populates a running instance with a suite of real artifacts.
//!
//! This exists because there was no way to get a realistic amount of content into an instance
//! without doing it by hand. Browsing, search, badges, `maven-metadata.xml` merging and the project
//! pages all behave differently with one artifact than with forty, and most of the defects found
//! during this work only showed up with more than one version present.
//!
//! Deploys go over the real protocols — `PUT` of a jar with its sibling `.sha1`/`.md5`, and an npm
//! packument with a base64 `_attachments` entry — rather than writing to the database, so a seed
//! run is also a smoke test of the deploy paths.

use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{Client, StatusCode};
use serde_json::json;
use tracing::{info, warn};

pub mod artifacts;
pub mod config;

use config::{
    CargoCrate, DockerImage, MavenProject, NpmPackage, SeedAuth, SeedConfig, SeedRepository,
    SeedStorage,
};

pub struct Seeder {
    client: Client,
    base: String,
    auth: SeedAuth,
    /// Set after signing in with a username and password.
    ///
    /// The API routes accept a session or a bearer token but *not* basic auth, while the deploy
    /// endpoints accept basic (that is what `mvn deploy` sends). So a password-configured seed has
    /// to hold both: a session for creating storages and repositories, and basic for the uploads.
    session: Option<String>,
    /// Counts what happened, so the run ends with something to assert on.
    pub summary: Summary,
}

#[derive(Debug, Default)]
pub struct Summary {
    pub storages_created: usize,
    pub repositories_created: usize,
    pub files_uploaded: usize,
    pub packages_published: usize,
    pub skipped: usize,
}

impl Seeder {
    pub fn new(config: &SeedConfig) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent(concat!("nitro_repo-seed/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self {
            client,
            base: config.url.trim_end_matches('/').to_owned(),
            auth: config.auth.clone(),
            session: None,
            summary: Summary::default(),
        })
    }

    /// Credentials for the deploy endpoints, which is what a Maven or npm client would send.
    fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth {
            SeedAuth::Token { token } => request.bearer_auth(token),
            SeedAuth::Basic { username, password } => request.basic_auth(username, Some(password)),
        }
    }

    /// Credentials for `/api/...`, which refuses basic auth.
    fn authorize_api(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match (&self.auth, &self.session) {
            (SeedAuth::Token { token }, _) => request.bearer_auth(token),
            (_, Some(session)) => request.header("Authorization", format!("Session {session}")),
            // Unreachable in practice: `sign_in` runs before anything calls this, and fails loudly.
            (SeedAuth::Basic { .. }, None) => request,
        }
    }

    /// Exchanges a username and password for a session.
    ///
    /// A token-configured seed skips this — a token already works everywhere.
    async fn sign_in(&mut self) -> anyhow::Result<()> {
        let SeedAuth::Basic { username, password } = &self.auth else {
            return Ok(());
        };

        let response = self
            .client
            .post(format!("{}/api/user/login", self.base))
            .json(&json!({ "email_or_username": username, "password": password }))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!(
                "Could not sign in as `{username}`: {status}. Check the credentials in the config, \
                 and that the instance has been installed."
            );
        }

        let body: serde_json::Value = response.json().await?;
        let Some(session) = body
            .get("session")
            .and_then(|value| value.get("session_id"))
            .and_then(|value| value.as_str())
        else {
            anyhow::bail!("Signed in, but the response carried no session id");
        };

        self.session = Some(session.to_owned());
        Ok(())
    }

    pub async fn run(&mut self, config: &SeedConfig) -> anyhow::Result<()> {
        self.check_reachable().await?;
        self.sign_in().await?;

        for storage in &config.storages {
            self.ensure_storage(storage).await?;
        }
        for repository in &config.repositories {
            self.ensure_repository(repository).await?;
        }
        for project in &config.maven {
            self.deploy_maven(project).await?;
        }
        for package in &config.npm {
            self.publish_npm(package).await?;
        }
        for krate in &config.cargo {
            self.publish_crate(krate).await?;
        }
        for image in &config.docker {
            self.push_image(image).await?;
        }

        Ok(())
    }

    /// Fails early with something readable. Without this, a wrong URL surfaces as a connection
    /// error on the first deploy, several steps in.
    async fn check_reachable(&self) -> anyhow::Result<()> {
        let response = self
            .client
            .get(format!("{}/api/info", self.base))
            .send()
            .await
            .map_err(|error| {
                anyhow::anyhow!("Could not reach {} — is it running? ({error})", self.base)
            })?;

        if !response.status().is_success() {
            anyhow::bail!(
                "{}/api/info returned {}. Check the URL points at a nitro_repo instance.",
                self.base,
                response.status()
            );
        }
        Ok(())
    }

    async fn ensure_storage(&mut self, storage: &SeedStorage) -> anyhow::Result<()> {
        let existing = self
            .authorize_api(self.client.get(format!("{}/api/storage/list", self.base)))
            .send()
            .await?;

        if existing.status() == StatusCode::UNAUTHORIZED
            || existing.status() == StatusCode::FORBIDDEN
        {
            anyhow::bail!("Not authorised to list storages. Check the credentials in the config.");
        }

        let storages: Vec<serde_json::Value> = existing.json().await.unwrap_or_default();
        if storages
            .iter()
            .any(|value| value.get("name").and_then(|n| n.as_str()) == Some(&storage.name))
        {
            info!(name = %storage.name, "Storage already exists");
            self.summary.skipped += 1;
            return Ok(());
        }

        let response = self
            .authorize_api(
                self.client
                    .post(format!(
                        "{}/api/storage/new/{}",
                        self.base, storage.storage_type
                    ))
                    .json(&json!({
                        "name": storage.name,
                        "config": { "type": storage.storage_type, "settings": storage.settings },
                    })),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Could not create storage `{}`: {status} {body}",
                storage.name
            );
        }

        info!(name = %storage.name, "Created storage");
        self.summary.storages_created += 1;
        Ok(())
    }

    async fn ensure_repository(&mut self, repository: &SeedRepository) -> anyhow::Result<()> {
        // The find-by-name endpoint answers 404 when there is no such repository, which is the
        // cheapest existence check available and does not need the whole list.
        let lookup = self
            .authorize_api(self.client.get(format!(
                "{}/api/repository/find-id/{}/{}",
                self.base, repository.storage, repository.name
            )))
            .send()
            .await?;

        if lookup.status().is_success() {
            info!(name = %repository.name, "Repository already exists");
            self.summary.skipped += 1;
            return Ok(());
        }

        let storages: Vec<serde_json::Value> = self
            .authorize_api(self.client.get(format!("{}/api/storage/list", self.base)))
            .send()
            .await?
            .json()
            .await
            .unwrap_or_default();

        let Some(storage_id) = storages
            .iter()
            .find(|value| value.get("name").and_then(|n| n.as_str()) == Some(&repository.storage))
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str())
        else {
            anyhow::bail!(
                "Repository `{}` wants storage `{}`, which does not exist. Add it under [[storages]].",
                repository.name,
                repository.storage
            );
        };

        let response = self
            .authorize_api(
                self.client
                    .post(format!(
                        "{}/api/repository/new/{}",
                        self.base, repository.repository_type
                    ))
                    // `configs` carries the type-specific config keyed by config name. Both
                    // repository types currently have only a `Hosted` variant; a proxy would be
                    // configured after creation.
                    .json(&json!({
                        "name": repository.name,
                        "storage": storage_id,
                        "configs": {
                            repository.repository_type.clone(): { "type": "Hosted" },
                        },
                    })),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Could not create repository `{}`: {status} {body}",
                repository.name
            );
        }

        let created: serde_json::Value = response.json().await.unwrap_or_default();

        // Visibility is not part of the create request, so it is a follow-up. A seeded instance
        // should be browsable without signing in, which is the point of defaulting to Public.
        if let Some(id) = created.get("id").and_then(|value| value.as_str())
            && repository.visibility != "Private"
        {
            let updated = self
                .authorize_api(
                    self.client
                        .put(format!("{}/api/repository/{id}", self.base))
                        .json(&json!({ "visibility": repository.visibility })),
                )
                .send()
                .await?;
            if !updated.status().is_success() {
                warn!(
                    name = %repository.name,
                    status = %updated.status(),
                    "Created the repository but could not set its visibility"
                );
            }
        }

        info!(name = %repository.name, "Created repository");
        self.summary.repositories_created += 1;
        Ok(())
    }

    /// Uploads one file plus the `.sha1` and `.md5` a Maven client sends beside it.
    ///
    /// The checksums are not decoration — the server verifies an uploaded checksum against what it
    /// computed, so getting them wrong here is caught immediately rather than producing a repository
    /// full of artifacts nothing can validate.
    async fn put_maven_file(
        &mut self,
        repository: &str,
        path: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> anyhow::Result<()> {
        let sha1 = artifacts::sha1_hex(&data);
        let md5 = artifacts::md5_hex(&data);

        self.put_raw(repository, path, data, content_type).await?;
        self.put_raw(
            repository,
            &format!("{path}.sha1"),
            sha1.into_bytes(),
            "text/plain",
        )
        .await?;
        self.put_raw(
            repository,
            &format!("{path}.md5"),
            md5.into_bytes(),
            "text/plain",
        )
        .await?;
        Ok(())
    }

    async fn put_raw(
        &mut self,
        repository: &str,
        path: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> anyhow::Result<()> {
        let url = format!("{}/repositories/{}/{}", self.base, repository, path);
        let response = self
            .authorize(
                self.client
                    .put(&url)
                    .header("Content-Type", content_type)
                    .body(data),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("PUT {url} failed: {status} {body}");
        }

        self.summary.files_uploaded += 1;
        Ok(())
    }

    async fn deploy_maven(&mut self, project: &MavenProject) -> anyhow::Result<()> {
        let group_path = project.group_id.replace('.', "/");

        for version in &project.versions {
            let base = format!("{group_path}/{}/{version}", project.artifact_id);

            // A real `mvn deploy` of a snapshot does not upload `artifact-1.2.0-SNAPSHOT.jar` — it
            // uploads a timestamped build, `artifact-1.2.0-20260801.224723-1.jar`, and the
            // repository's snapshot metadata points at the newest. The plain name is what a local
            // `mvn install` produces, and deploying that leaves the snapshot metadata with nothing
            // to describe. Two builds go up so `<snapshotVersions>` has more than one to pick from.
            let builds: Vec<Option<String>> = if version.ends_with("-SNAPSHOT") {
                let stamp = chrono::Local::now().format("%Y%m%d.%H%M%S").to_string();
                vec![Some(format!("{stamp}-1")), Some(format!("{stamp}-2"))]
            } else {
                vec![None]
            };

            for build in &builds {
                let file_version = match build {
                    Some(build) => format!("{}-{build}", version.trim_end_matches("-SNAPSHOT")),
                    None => version.clone(),
                };
                self.deploy_maven_build(project, version, &base, &file_version)
                    .await?;
            }

            info!(
                repository = %project.repository,
                artifact = %format!("{}:{}:{version}", project.group_id, project.artifact_id),
                builds = builds.len(),
                "Deployed"
            );
        }

        Ok(())
    }

    /// Uploads one build of one version: the POM, the main jar, and any classified jars.
    async fn deploy_maven_build(
        &mut self,
        project: &MavenProject,
        version: &str,
        base: &str,
        file_version: &str,
    ) -> anyhow::Result<()> {
        let stem = format!("{}-{file_version}", project.artifact_id);

        // The POM carries the logical version (`1.2.0-SNAPSHOT`) even when the file it is stored
        // under is timestamped. That is what Maven writes, and what the server reads to decide
        // which project a deploy belongs to.
        let pom = artifacts::pom(project, version);
        self.put_maven_file(
            &project.repository,
            &format!("{base}/{stem}.pom"),
            pom.into_bytes(),
            "application/xml",
        )
        .await?;

        let jar = artifacts::jar(project, version, None)?;
        self.put_maven_file(
            &project.repository,
            &format!("{base}/{stem}.jar"),
            jar,
            "application/java-archive",
        )
        .await?;

        for classifier in &project.classifiers {
            let jar = artifacts::jar(project, version, Some(classifier))?;
            self.put_maven_file(
                &project.repository,
                &format!("{base}/{stem}-{classifier}.jar"),
                jar,
                "application/java-archive",
            )
            .await?;
        }

        Ok(())
    }

    async fn publish_npm(&mut self, package: &NpmPackage) -> anyhow::Result<()> {
        let mut dist_tags = package.dist_tags.clone();

        for version in &package.versions {
            let tarball =
                artifacts::npm_tarball(&package.name, version, package.description.as_deref())?;
            let integrity = artifacts::integrity(&tarball);
            let shasum = artifacts::shasum(&tarball);

            // npm names the attachment by the *unscoped* filename, and the tarball URL by the full
            // package path. Getting these the wrong way round is exactly the mismatch that made
            // scoped packages unusable before Phase 3.
            let unscoped = package.name.rsplit('/').next().unwrap_or(&package.name);
            let file_name = format!("{unscoped}-{version}.tgz");
            let tarball_url = format!(
                "{}/repositories/{}/{}/-/{file_name}",
                self.base, package.repository, package.name
            );

            let mut manifest =
                artifacts::package_json(&package.name, version, package.description.as_deref());
            if let Some(object) = manifest.as_object_mut() {
                object.insert(
                    "_id".to_owned(),
                    json!(format!("{}@{version}", package.name)),
                );
                object.insert(
                    "dist".to_owned(),
                    json!({
                        "integrity": integrity,
                        "shasum": shasum,
                        "tarball": tarball_url,
                    }),
                );
                object.insert("_nodeVersion".to_owned(), json!("20.0.0"));
                object.insert("_npmVersion".to_owned(), json!("10.0.0"));
                object.insert("readme".to_owned(), json!(format!("# {}", package.name)));
                object.insert("readmeFilename".to_owned(), json!("README.md"));
            }

            let mut versions = BTreeMap::new();
            versions.insert(version.clone(), manifest);

            let mut attachments = BTreeMap::new();
            attachments.insert(
                file_name,
                json!({
                    "content_type": "application/octet-stream",
                    "data": STANDARD.encode(&tarball),
                    "length": tarball.len(),
                }),
            );

            let body = json!({
                "_id": package.name,
                "name": package.name,
                "description": package.description,
                "versions": versions,
                "_attachments": attachments,
            });

            let url = format!(
                "{}/repositories/{}/{}",
                self.base, package.repository, package.name
            );
            let response = self
                .authorize(
                    self.client
                        .put(&url)
                        .header("npm-command", "publish")
                        .json(&body),
                )
                .send()
                .await?;

            let status = response.status();
            if status == StatusCode::CONFLICT {
                // Re-running a seed is expected to be safe; a version that is already there is not
                // an error, and overwriting it is exactly what the publish path refuses to do.
                warn!(package = %package.name, %version, "Already published, leaving it alone");
                self.summary.skipped += 1;
            } else if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!(
                    "Publishing {}@{version} failed: {status} {body}",
                    package.name
                );
            } else {
                info!(package = %package.name, %version, "Published");
                self.summary.packages_published += 1;
            }

            dist_tags
                .entry("latest".to_owned())
                .or_insert(version.clone());
            // Later versions win the implicit `latest`, matching what npm does.
            if !package.dist_tags.contains_key("latest") {
                dist_tags.insert("latest".to_owned(), version.clone());
            }
        }

        for (tag, version) in &dist_tags {
            let url = format!(
                "{}/repositories/{}/-/package/{}/dist-tags/{tag}",
                self.base, package.repository, package.name
            );
            let response = self
                .authorize(self.client.put(&url).json(version))
                .send()
                .await?;
            if !response.status().is_success() {
                warn!(
                    package = %package.name,
                    %tag,
                    status = %response.status(),
                    "Could not set dist-tag"
                );
            }
        }

        Ok(())
    }

    /// `cargo publish`: a `PUT` of two length-prefixed frames to `/api/v1/crates/new`.
    async fn publish_crate(&mut self, krate: &CargoCrate) -> anyhow::Result<()> {
        for version in &krate.versions {
            let crate_file =
                artifacts::crate_file(&krate.name, version, krate.description.as_deref())?;
            let metadata = json!({
                "name": krate.name,
                "vers": version,
                "deps": [],
                "features": {},
                "authors": ["nitro_repo seed"],
                "description": krate.description.clone().unwrap_or_else(|| "Seeded by nitro_repo".to_owned()),
                "keywords": [],
                "categories": [],
                "license": "MIT",
            });
            let body = artifacts::cargo_publish_body(&metadata, &crate_file);

            let url = format!(
                "{}/repositories/{}/api/v1/crates/new",
                self.base, krate.repository
            );
            let response = self
                .authorize(
                    self.client
                        .put(&url)
                        .header("Content-Type", "application/octet-stream")
                        .body(body),
                )
                .send()
                .await?;

            let status = response.status();
            if status == StatusCode::CONFLICT {
                warn!(krate = %krate.name, %version, "Already published, leaving it alone");
                self.summary.skipped += 1;
            } else if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!(
                    "Publishing {}@{version} failed: {status} {body}",
                    krate.name
                );
            } else {
                info!(krate = %krate.name, %version, "Published");
                self.summary.packages_published += 1;
            }
        }
        Ok(())
    }

    /// `docker push`: each blob through the three-step upload, then the manifest.
    ///
    /// The image name carries the `{storage}/{repository}` prefix because a seeded registry has no
    /// hostname of its own — this is exactly what a client would send to
    /// `docker push localhost:6742/local/docker/example:1.0`.
    async fn push_image(&mut self, image: &DockerImage) -> anyhow::Result<()> {
        let name = format!("{}/{}", image.repository, image.name);

        for tag in &image.tags {
            // One layer per tag, so the two tags are genuinely different images rather than the
            // same manifest under two names — which would not exercise re-tagging.
            let layer = artifacts::oci_layer("hello.txt", &format!("{}:{tag}", image.name))?;
            let config = artifacts::oci_config(&layer);
            let manifest = artifacts::oci_manifest(&config, &layer.gzipped);

            for blob in [&layer.gzipped, &config] {
                self.push_blob(&name, blob).await?;
            }

            let url = format!("{}/v2/{name}/manifests/{tag}", self.base);
            let response = self
                .authorize(
                    self.client
                        .put(&url)
                        .header("Content-Type", "application/vnd.oci.image.manifest.v1+json")
                        .body(manifest),
                )
                .send()
                .await?;

            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!("Pushing {name}:{tag} failed: {status} {body}");
            }
            info!(image = %name, %tag, "Pushed");
            self.summary.packages_published += 1;
        }
        Ok(())
    }

    async fn push_blob(&mut self, name: &str, blob: &[u8]) -> anyhow::Result<()> {
        let digest = artifacts::oci_digest(blob);

        // A blob already present is the common case on a re-run, and skipping it is what a real
        // client does too.
        let head = self
            .authorize(
                self.client
                    .head(format!("{}/v2/{name}/blobs/{digest}", self.base)),
            )
            .send()
            .await?;
        if head.status().is_success() {
            self.summary.skipped += 1;
            return Ok(());
        }

        let started = self
            .authorize(
                self.client
                    .post(format!("{}/v2/{name}/blobs/uploads/", self.base)),
            )
            .send()
            .await?;
        if !started.status().is_success() {
            let status = started.status();
            let body = started.text().await.unwrap_or_default();
            anyhow::bail!("Opening a blob upload for {name} failed: {status} {body}");
        }
        let location = started
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow::anyhow!("The registry opened an upload with no Location"))?;
        // Relative, as the registry emits it, so it is resolved against the instance base here.
        let location = format!("{}{location}", self.base);

        let response = self
            .authorize(
                self.client
                    .put(format!("{location}?digest={digest}"))
                    .body(blob.to_vec()),
            )
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Committing a blob for {name} failed: {status} {body}");
        }
        self.summary.files_uploaded += 1;
        Ok(())
    }
}

/// Entry point for the `seed` subcommand.
pub fn seed(config_path: PathBuf, write_example: bool) -> anyhow::Result<()> {
    if write_example {
        let example = SeedConfig::example();
        let rendered = toml::to_string_pretty(&example)?;
        std::fs::write(&config_path, rendered)?;
        println!("Wrote an example seed config to {}", config_path.display());
        return Ok(());
    }

    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist. Run `nitro_repo seed --config {} --write-example` to create one.",
            config_path.display(),
            config_path.display()
        );
    }

    let config: SeedConfig = toml::from_str(&std::fs::read_to_string(&config_path)?)?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let mut seeder = Seeder::new(&config)?;
        seeder.run(&config).await?;

        let summary = &seeder.summary;
        println!(
            "Seeded {}: {} storage(s), {} repositor(ies), {} file(s), {} package version(s){}",
            config.url,
            summary.storages_created,
            summary.repositories_created,
            summary.files_uploaded,
            summary.packages_published,
            if summary.skipped > 0 {
                format!(", {} already present", summary.skipped)
            } else {
                String::new()
            }
        );
        Ok(())
    })
}
