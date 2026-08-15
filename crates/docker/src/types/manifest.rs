//! Manifests and indexes, parsed only as far as this registry needs them.
//!
//! A manifest is stored and re-served as the exact bytes that were pushed — its digest is over
//! those bytes, so re-serialising it would change its identity. These types exist to answer two
//! questions about a manifest before it is accepted: which blobs must already exist, and what does
//! it refer to.

use serde::Deserialize;

use super::digest::Digest;

pub const MEDIA_TYPE_DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
pub const MEDIA_TYPE_DOCKER_MANIFEST_LIST: &str =
    "application/vnd.docker.distribution.manifest.list.v2+json";
pub const MEDIA_TYPE_OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub const MEDIA_TYPE_OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";

/// The default a client is assumed to have meant when it sends no `Content-Type`.
pub const DEFAULT_MANIFEST_MEDIA_TYPE: &str = MEDIA_TYPE_DOCKER_MANIFEST;

/// Whether a media type names a multi-platform index rather than a single image.
pub fn is_index(media_type: &str) -> bool {
    matches!(
        media_type,
        MEDIA_TYPE_DOCKER_MANIFEST_LIST | MEDIA_TYPE_OCI_INDEX
    )
}

/// One entry in a manifest's `layers`, or its `config`.
#[derive(Debug, Clone, Deserialize)]
pub struct Descriptor {
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    pub digest: Digest,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub platform: Option<Platform>,
    /// Set on a foreign//non-distributable layer, which by definition is not in this registry.
    #[serde(default)]
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Platform {
    #[serde(default)]
    pub architecture: Option<String>,
    #[serde(default)]
    pub os: Option<String>,
    #[serde(default)]
    pub variant: Option<String>,
}

/// The union of an image manifest and an index — enough of either to validate it.
///
/// One type rather than two because the fields do not overlap: an image manifest has `config` and
/// `layers`, an index has `manifests`, and which one applies is decided by the media type.
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    #[serde(rename = "schemaVersion", default)]
    pub schema_version: u32,
    #[serde(rename = "mediaType", default)]
    pub media_type: Option<String>,
    #[serde(rename = "artifactType", default)]
    pub artifact_type: Option<String>,
    #[serde(default)]
    pub config: Option<Descriptor>,
    #[serde(default)]
    pub layers: Vec<Descriptor>,
    #[serde(default)]
    pub manifests: Vec<Descriptor>,
    /// The OCI referrers relationship: this manifest is *about* the manifest it names.
    #[serde(default)]
    pub subject: Option<Descriptor>,
}

impl Manifest {
    /// The blobs that must already be in storage before this manifest can be accepted.
    ///
    /// A layer that carries `urls` is a foreign layer — it lives somewhere else by design, and
    /// requiring it here would make every Windows base image unpushable.
    pub fn required_blobs(&self) -> Vec<&Digest> {
        self.config
            .iter()
            .chain(self.layers.iter())
            .filter(|descriptor| descriptor.urls.is_empty())
            .map(|descriptor| &descriptor.digest)
            .collect()
    }

    /// The child manifests an index points at.
    pub fn child_manifests(&self) -> Vec<&Digest> {
        self.manifests
            .iter()
            .map(|descriptor| &descriptor.digest)
            .collect()
    }

    /// The platforms an index covers, formatted for display.
    pub fn platforms(&self) -> Vec<String> {
        self.manifests
            .iter()
            .filter_map(|descriptor| descriptor.platform.as_ref())
            .map(|platform| {
                let os = platform.os.as_deref().unwrap_or("unknown");
                let architecture = platform.architecture.as_deref().unwrap_or("unknown");
                match platform.variant.as_deref() {
                    Some(variant) => format!("{os}/{architecture}/{variant}"),
                    None => format!("{os}/{architecture}"),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{Manifest, is_index};

    const CONFIG: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const LAYER: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
    const FOREIGN: &str = "sha256:3333333333333333333333333333333333333333333333333333333333333333";

    #[test]
    fn an_image_manifest_lists_its_config_and_layers() {
        let manifest: Manifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 2,
            "mediaType": super::MEDIA_TYPE_OCI_MANIFEST,
            "config": { "mediaType": "application/vnd.oci.image.config.v1+json", "digest": CONFIG, "size": 7 },
            "layers": [
                { "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip", "digest": LAYER, "size": 32 },
            ],
        }))
        .unwrap();

        let required: Vec<String> = manifest
            .required_blobs()
            .into_iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(required, vec![CONFIG.to_owned(), LAYER.to_owned()]);
    }

    /// A foreign layer lives outside this registry on purpose; demanding it would make every
    /// Windows base image impossible to push.
    #[test]
    fn a_foreign_layer_is_not_required_to_be_present() {
        let manifest: Manifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 2,
            "config": { "digest": CONFIG, "size": 7 },
            "layers": [
                { "digest": LAYER, "size": 32 },
                { "digest": FOREIGN, "size": 99, "urls": ["https://example.com/layer"] },
            ],
        }))
        .unwrap();

        let required: Vec<String> = manifest
            .required_blobs()
            .into_iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(required, vec![CONFIG.to_owned(), LAYER.to_owned()]);
    }

    #[test]
    fn an_index_lists_its_children_and_platforms() {
        let manifest: Manifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 2,
            "mediaType": super::MEDIA_TYPE_OCI_INDEX,
            "manifests": [
                { "digest": CONFIG, "size": 1, "platform": { "os": "linux", "architecture": "amd64" } },
                { "digest": LAYER, "size": 1, "platform": { "os": "linux", "architecture": "arm64", "variant": "v8" } },
            ],
        }))
        .unwrap();

        assert!(manifest.required_blobs().is_empty());
        assert_eq!(manifest.child_manifests().len(), 2);
        assert_eq!(
            manifest.platforms(),
            vec!["linux/amd64".to_owned(), "linux/arm64/v8".to_owned()]
        );
    }

    #[test]
    fn index_media_types_are_recognised() {
        assert!(is_index(super::MEDIA_TYPE_OCI_INDEX));
        assert!(is_index(super::MEDIA_TYPE_DOCKER_MANIFEST_LIST));
        assert!(!is_index(super::MEDIA_TYPE_OCI_MANIFEST));
        assert!(!is_index(super::MEDIA_TYPE_DOCKER_MANIFEST));
    }

    #[test]
    fn a_manifest_with_a_subject_keeps_it() {
        let manifest: Manifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 2,
            "config": { "digest": CONFIG, "size": 7 },
            "layers": [],
            "artifactType": "application/vnd.example.sbom",
            "subject": { "digest": LAYER, "size": 1 },
        }))
        .unwrap();

        assert_eq!(
            manifest.subject.unwrap().digest.to_string(),
            LAYER.to_owned()
        );
        assert_eq!(
            manifest.artifact_type.as_deref(),
            Some("application/vnd.example.sbom")
        );
    }

    /// A digest that will not parse must fail the manifest, not be silently dropped — a manifest
    /// with an unverifiable layer reference is not one this registry can serve.
    #[test]
    fn a_manifest_with_an_unparseable_digest_is_refused() {
        let result: Result<Manifest, _> = serde_json::from_value(serde_json::json!({
            "schemaVersion": 2,
            "layers": [{ "digest": "not-a-digest", "size": 1 }],
        }));
        assert!(result.is_err());
    }
}
