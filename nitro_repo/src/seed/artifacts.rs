//! Generates the actual bytes a client would upload.
//!
//! These are real artifacts, not placeholder blobs: a jar is a real zip with a manifest, a POM is
//! real XML, and an npm tarball is a real gzipped tar with the `package/` prefix npm expects. That
//! matters — a seeded instance should be indistinguishable from one that real `mvn deploy` and
//! `npm publish` runs produced, or it will not exercise the code paths those clients hit.

use std::io::Write;

use flate2::{Compression, write::GzEncoder};
use sha1::Sha1;
use sha2::{Digest, Sha512};
use zip::{ZipWriter, write::SimpleFileOptions};

use super::config::MavenProject;

/// A POM for one version of a project.
pub fn pom(project: &MavenProject, version: &str) -> String {
    let mut dependencies = String::new();
    for dependency in &project.dependencies {
        let parts: Vec<&str> = dependency.split(':').collect();
        if parts.len() != 3 {
            continue;
        }
        dependencies.push_str(&format!(
            "        <dependency>\n            <groupId>{}</groupId>\n            <artifactId>{}</artifactId>\n            <version>{}</version>\n        </dependency>\n",
            escape(parts[0]),
            escape(parts[1]),
            escape(parts[2]),
        ));
    }

    let description = project
        .description
        .as_deref()
        .map(|value| format!("    <description>{}</description>\n", escape(value)))
        .unwrap_or_default();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>{group}</groupId>
    <artifactId>{artifact}</artifactId>
    <version>{version}</version>
    <packaging>jar</packaging>
    <name>{artifact}</name>
{description}    <dependencies>
{dependencies}    </dependencies>
</project>
"#,
        group = escape(&project.group_id),
        artifact = escape(&project.artifact_id),
        version = escape(version),
        description = description,
        dependencies = dependencies,
    )
}

/// A jar: a real zip holding a manifest and a marker file, so it opens in any zip tool.
pub fn jar(
    project: &MavenProject,
    version: &str,
    classifier: Option<&str>,
) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default();

        zip.start_file("META-INF/MANIFEST.MF", options)?;
        write!(
            zip,
            "Manifest-Version: 1.0\r\nCreated-By: nitro_repo seed\r\nImplementation-Title: {}\r\nImplementation-Version: {}\r\n\r\n",
            project.artifact_id, version
        )?;

        let name = match classifier {
            // Distinct content per classifier, so the sources and javadoc jars are not byte-identical
            // to the main one — identical bytes would hide any bug that mixes them up.
            Some(classifier) => format!("{}-{}.txt", project.artifact_id, classifier),
            None => format!("{}.txt", project.artifact_id),
        };
        zip.start_file(name, options)?;
        writeln!(
            zip,
            "{}:{}:{}{}",
            project.group_id,
            project.artifact_id,
            version,
            classifier
                .map(|value| format!(":{value}"))
                .unwrap_or_default()
        )?;

        zip.finish()?;
    }
    Ok(buffer)
}

/// A `package.json` for one version of an npm package.
pub fn package_json(name: &str, version: &str, description: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "version": version,
        "description": description.unwrap_or("Seeded by nitro_repo"),
        "main": "index.js",
        "license": "MIT",
        "scripts": { "test": "echo \"no tests\" && exit 0" },
    })
}

/// An npm tarball: gzipped tar, every entry under `package/`, which is what npm produces and what
/// every client expects when it unpacks one.
pub fn npm_tarball(
    name: &str,
    version: &str,
    description: Option<&str>,
) -> anyhow::Result<Vec<u8>> {
    let manifest = serde_json::to_vec_pretty(&package_json(name, version, description))?;
    let readme = format!(
        "# {name}\n\n{}\n\nVersion {version}.\n",
        description.unwrap_or("")
    );
    let index = format!("module.exports = {{ name: {name:?}, version: {version:?} }};\n");

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        append(&mut builder, "package/package.json", &manifest)?;
        append(&mut builder, "package/README.md", readme.as_bytes())?;
        append(&mut builder, "package/index.js", index.as_bytes())?;
        builder.finish()?;
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes)?;
    Ok(encoder.finish()?)
}

fn append<W: Write>(builder: &mut tar::Builder<W>, path: &str, data: &[u8]) -> anyhow::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    // A fixed mtime keeps the tarball byte-reproducible, so re-running a seed produces the same
    // integrity hash rather than a new one every time.
    header.set_mtime(0);
    header.set_cksum();
    builder.append_data(&mut header, path, data)?;
    Ok(())
}

/// The `integrity` string npm sends: SSRI, base64 of the raw sha512 digest.
pub fn integrity(data: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let digest = Sha512::digest(data);
    format!("sha512-{}", STANDARD.encode(digest))
}

/// The `shasum` npm sends alongside it: hex sha1.
pub fn shasum(data: &[u8]) -> String {
    let digest = Sha1::digest(data);
    hex(&digest)
}

pub fn sha1_hex(data: &[u8]) -> String {
    hex(&Sha1::digest(data))
}

pub fn md5_hex(data: &[u8]) -> String {
    hex(&md5::Md5::digest(data))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, byte| {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
        out
    })
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> MavenProject {
        MavenProject {
            repository: "local/releases".to_owned(),
            group_id: "dev.kingtux".to_owned(),
            artifact_id: "tms".to_owned(),
            versions: vec!["1.0.0".to_owned()],
            description: Some("Example".to_owned()),
            classifiers: vec![],
            dependencies: vec!["org.slf4j:slf4j-api:2.0.13".to_owned()],
        }
    }

    /// The server parses what this writes, so it has to be a POM `maven-rs` accepts — not merely
    /// well-formed XML.
    #[test]
    fn the_generated_pom_parses() {
        let text = pom(&project(), "1.0.0");
        // Parsed exactly the way the server parses an uploaded POM (`maven/utils.rs`), so this
        // proves the deploy path would accept it — not merely that it is well-formed XML.
        let parsed: maven_rs::pom::Pom =
            maven_rs::quick_xml::de::from_str(&text).expect("the seed's POM should parse");
        assert_eq!(parsed.group_id.as_deref(), Some("dev.kingtux"));
        assert_eq!(parsed.artifact_id, "tms");
        assert_eq!(parsed.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn the_generated_jar_is_a_readable_zip() {
        let bytes = jar(&project(), "1.0.0", None).expect("builds");
        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("should be a valid zip");
        assert!(archive.by_name("META-INF/MANIFEST.MF").is_ok());
    }

    /// Classified jars must differ from the main one, or a bug that serves the wrong classifier
    /// would be invisible.
    #[test]
    fn classified_jars_differ_from_the_main_jar() {
        let main = jar(&project(), "1.0.0", None).unwrap();
        let sources = jar(&project(), "1.0.0", Some("sources")).unwrap();
        assert_ne!(main, sources);
    }

    #[test]
    fn the_npm_tarball_unpacks_with_a_package_prefix() {
        let bytes = npm_tarball("@nitro/example", "1.0.0", Some("Example")).expect("builds");
        let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(bytes));
        let mut archive = tar::Archive::new(decoder);

        let paths: Vec<String> = archive
            .entries()
            .expect("entries")
            .map(|entry| entry.unwrap().path().unwrap().display().to_string())
            .collect();

        assert!(
            paths.contains(&"package/package.json".to_owned()),
            "{paths:?}"
        );
        assert!(paths.contains(&"package/index.js".to_owned()), "{paths:?}");
    }

    /// The server verifies both of these on publish, so a seed that computed them differently would
    /// be rejected — and the failure would read as a server bug rather than a seed bug. Pinned
    /// against known digests of a fixed input rather than against a re-run of the same code, which
    /// would pass no matter what the code did.
    #[test]
    fn integrity_and_shasum_match_their_algorithms() {
        // The canonical SHA-1 and SHA-512 of the empty string.
        assert_eq!(shasum(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(
            integrity(b""),
            "sha512-z4PhNX7vuL3xVChQ1m2AB9Yg5AULVxXcg/SpIdNs6c5H0NE8XYXysP+DGNKHfuwvY7kxvUdBeoGlODJ6+SfaPg=="
        );
    }

    #[test]
    fn checksum_helpers_agree_with_known_digests() {
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    /// A tarball must be byte-identical across runs, so re-seeding does not produce a new integrity
    /// hash for content that has not changed.
    #[test]
    fn tarballs_are_reproducible() {
        let first = npm_tarball("@nitro/example", "1.0.0", Some("Example")).unwrap();
        let second = npm_tarball("@nitro/example", "1.0.0", Some("Example")).unwrap();
        assert_eq!(first, second);
    }
}
