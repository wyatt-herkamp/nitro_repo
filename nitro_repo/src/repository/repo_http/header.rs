use http::HeaderName;
use tracing::{debug, instrument, trace, warn};

use crate::{
    error::BadRequestErrors, repository::RepositoryHandlerError, utils::headers::HeaderValueExt,
};

use super::RepositoryRequest;
/// Nitro Repo Deploy is a custom header used to identify that the request is coming from a Nitro Repo Deploy Client
pub const NITRO_REPO_DEPLOY_HEADER: HeaderName = HeaderName::from_static("x-nitro-repo-deploy");
/// Header Structure for Nitro Repo Deploy
///
/// The value should be formatted as follows: `{Repository Type} {Version}`
///
/// Not all repositories will have a custom deploy system.
#[derive(Debug)]
pub struct NitroRepoDeployHeaderValue {
    pub repository_type: String,
    pub version: u8,
}
impl TryFrom<String> for NitroRepoDeployHeaderValue {
    type Error = BadRequestErrors;
    #[instrument(name = "NitroRepoDeployHeaderValue::try_from")]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let params: Vec<_> = value.trim().split(" ").collect();
        if params.len() != 2 {
            warn!(?value, "Invalid Nitro Repo Deploy Header Value");
            return Err(BadRequestErrors::Other(format!(
                "Invalid Nitro Repo Deploy Header Value: {}",
                value
            )));
        }
        let repository_type = params[0].to_owned();
        let version: u8 = params[1].parse().map_err(|err| {
            warn!(?err, "Invalid Nitro Repo Deploy Header Value");
            BadRequestErrors::Other(format!("Invalid Nitro Repo Deploy Header Value: {}", value))
        })?;
        Ok(Self {
            repository_type,
            version,
        })
    }
}
impl RepositoryRequest {
    #[inline(always)]
    pub fn headers(&self) -> &http::HeaderMap {
        &self.parts.headers
    }
    #[instrument(skip(self))]
    pub fn get_nitro_repo_deploy_header(
        &self,
    ) -> Result<Option<NitroRepoDeployHeaderValue>, RepositoryHandlerError> {
        let Some(header) = self.headers().get(NITRO_REPO_DEPLOY_HEADER) else {
            debug!("No Nitro Repo Deploy Header Found");
            return Ok(None);
        };
        trace!(?header, "Found Nitro Repo Deploy Header");
        let header = header
            .to_string()
            .map_err(BadRequestErrors::from)?;
        debug!(?header, "Header Parsed to String");
        let value = NitroRepoDeployHeaderValue::try_from(header)?;
        Ok(Some(value))
    }
}
