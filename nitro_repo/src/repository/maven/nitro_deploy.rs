use nr_core::repository::project::VersionData;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum MavenFileType {
    Pom,
    Jar,
    SourcesJar,
    JavadocJar,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewNRMavenDeploy {
    pub group_id: String,
    pub artifact_id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub files: Vec<NewNRMavenDeployFile>,
    #[serde(default)]
    pub extra: Option<VersionData>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewNRMavenDeployFile {
    pub file_name: String,
    pub sha512: String,
    pub sha1: String,
    pub md5: String,
    pub maven_file_type: Option<MavenFileType>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewNrMavenDeploySuccessResponse {
    pub deploy_id: Uuid,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NRMavenPublishSuccessResponse {
    pub files: Vec<NRMavenPublishSuccessFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NRMavenPublishSuccessFile {
    pub name: String,
    pub path: String,
}
