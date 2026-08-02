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

/// A `.crate` file: a gzipped tar whose entries all sit under `{name}-{version}/`, which is the
/// layout `cargo package` produces and the one `cargo` expects when it unpacks a download.
pub fn crate_file(name: &str, version: &str, description: Option<&str>) -> anyhow::Result<Vec<u8>> {
    let prefix = format!("{name}-{version}");
    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"{version}\"\nedition = \"2021\"\n\
         description = \"{}\"\nlicense = \"MIT\"\n",
        description.unwrap_or("Seeded by nitro_repo")
    );
    let source = format!("pub fn name() -> &'static str {{ {name:?} }}\n");

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        append(
            &mut builder,
            &format!("{prefix}/Cargo.toml"),
            manifest.as_bytes(),
        )?;
        append(
            &mut builder,
            &format!("{prefix}/src/lib.rs"),
            source.as_bytes(),
        )?;
        builder.finish()?;
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes)?;
    Ok(encoder.finish()?)
}

/// The body `cargo publish` sends: two length-prefixed frames, metadata then the `.crate` file.
pub fn cargo_publish_body(metadata: &serde_json::Value, crate_file: &[u8]) -> Vec<u8> {
    let metadata = serde_json::to_vec(metadata).expect("metadata should serialise");
    let mut body = Vec::with_capacity(metadata.len() + crate_file.len() + 8);
    body.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    body.extend_from_slice(&metadata);
    body.extend_from_slice(&(crate_file.len() as u32).to_le_bytes());
    body.extend_from_slice(crate_file);
    body
}

pub fn sha256_hex(data: &[u8]) -> String {
    hex(&sha2::Sha256::digest(data))
}

/// The `algorithm:hex` digest a registry addresses content by.
pub fn oci_digest(data: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(data))
}

/// A gzipped-tar image layer holding one file, so a pushed image has real content in it.
pub struct OciLayer {
    /// What is uploaded as a blob, and what the manifest's layer descriptor names.
    pub gzipped: Vec<u8>,
    /// The digest of the *uncompressed* tar.
    ///
    /// This is what goes in the config's `rootfs.diff_ids`, and it is not the same as the blob's
    /// digest. A Docker daemon decompresses each layer on pull and checks the result against the
    /// diff id — using the compressed digest here makes the pull fail with `wrong diff id` after
    /// everything has already transferred.
    pub diff_id: String,
}

pub fn oci_layer(name: &str, contents: &str) -> anyhow::Result<OciLayer> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        append(&mut builder, name, contents.as_bytes())?;
        builder.finish()?;
    }
    let diff_id = oci_digest(&tar_bytes);

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes)?;
    Ok(OciLayer {
        gzipped: encoder.finish()?,
        diff_id,
    })
}

/// An OCI image config blob — the JSON document `docker inspect` shows.
pub fn oci_config(layer: &OciLayer) -> Vec<u8> {
    let config = serde_json::json!({
        "architecture": "amd64",
        "os": "linux",
        "config": { "Env": ["PATH=/usr/local/bin:/usr/bin:/bin"] },
        "rootfs": { "type": "layers", "diff_ids": [layer.diff_id] },
        "history": [{ "created_by": "nitro_repo seed" }],
    });
    serde_json::to_vec(&config).expect("the config should serialise")
}

/// An OCI image manifest pointing at a config and one layer.
///
/// Returned as bytes rather than as a `Value`, because a manifest's digest is over the exact bytes
/// that are sent — a caller that re-serialised it would compute a digest the registry disagrees
/// with.
pub fn oci_manifest(config: &[u8], layer: &[u8]) -> Vec<u8> {
    let manifest = serde_json::json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "digest": oci_digest(config),
            "size": config.len(),
        },
        "layers": [{
            "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
            "digest": oci_digest(layer),
            "size": layer.len(),
        }],
    });
    serde_json::to_vec(&manifest).expect("the manifest should serialise")
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
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// A layer's blob digest and its `diff_id` describe different bytes — compressed and not. A
    /// config that names the compressed digest passes every registry-side check and then fails the
    /// pull with `wrong diff id`, after the whole image has transferred.
    #[test]
    fn a_layers_diff_id_is_the_digest_of_the_uncompressed_tar() {
        let layer = oci_layer("hello.txt", "contents").expect("builds");
        assert_ne!(layer.diff_id, oci_digest(&layer.gzipped));

        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(
            &mut flate2::read::GzDecoder::new(std::io::Cursor::new(&layer.gzipped)),
            &mut decompressed,
        )
        .expect("the layer should be gzip");
        assert_eq!(layer.diff_id, oci_digest(&decompressed));

        // And the config repeats it verbatim.
        let config: serde_json::Value =
            serde_json::from_slice(&oci_config(&layer)).expect("the config should be JSON");
        assert_eq!(config["rootfs"]["diff_ids"][0], layer.diff_id);
    }

    #[test]
    fn the_manifest_points_at_the_config_and_the_compressed_layer() {
        let layer = oci_layer("hello.txt", "contents").unwrap();
        let config = oci_config(&layer);
        let manifest: serde_json::Value =
            serde_json::from_slice(&oci_manifest(&config, &layer.gzipped)).unwrap();

        assert_eq!(manifest["config"]["digest"], oci_digest(&config));
        assert_eq!(manifest["config"]["size"], config.len());
        assert_eq!(manifest["layers"][0]["digest"], oci_digest(&layer.gzipped));
        assert_eq!(manifest["layers"][0]["size"], layer.gzipped.len());
    }

    #[test]
    fn the_crate_file_unpacks_with_a_name_version_prefix() {
        let bytes = crate_file("example", "1.0.0", Some("Example")).expect("builds");
        let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(bytes));
        let mut archive = tar::Archive::new(decoder);

        let paths: Vec<String> = archive
            .entries()
            .expect("entries")
            .map(|entry| entry.unwrap().path().unwrap().display().to_string())
            .collect();

        assert!(
            paths.contains(&"example-1.0.0/Cargo.toml".to_owned()),
            "{paths:?}"
        );
        assert!(
            paths.contains(&"example-1.0.0/src/lib.rs".to_owned()),
            "{paths:?}"
        );
    }

    /// The frame lengths are little-endian u32s. Reading them the other way round would put the
    /// split in the wrong place, and the failure would surface as unparseable JSON.
    #[test]
    fn the_publish_body_is_two_length_prefixed_frames() {
        let metadata = serde_json::json!({ "name": "example", "vers": "1.0.0" });
        let body = cargo_publish_body(&metadata, b"crate bytes");

        let metadata_length = u32::from_le_bytes([body[0], body[1], body[2], body[3]]) as usize;
        let metadata_end = 4 + metadata_length;
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body[4..metadata_end]).unwrap(),
            metadata
        );

        let crate_length = u32::from_le_bytes([
            body[metadata_end],
            body[metadata_end + 1],
            body[metadata_end + 2],
            body[metadata_end + 3],
        ]) as usize;
        assert_eq!(crate_length, b"crate bytes".len());
        assert_eq!(&body[metadata_end + 4..], b"crate bytes");
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
