use serde::{Deserialize, Serialize};
use strum::EnumIter;
use url::Url;
use utoipa::ToSchema;

/// The AWS regions offered in the storage creation UI.
///
/// This is served by `GET /api/storage/s3/regions` and appears in the OpenAPI schema, so it stays
/// an enum even though the SDK models a region as an opaque string — the list is what makes the
/// dropdown possible. Anything not listed here can still be reached through [CustomRegion].

#[derive(Clone, Debug, Eq, Copy, PartialEq, Serialize, Deserialize, ToSchema, EnumIter)]
pub enum S3StorageRegion {
    /// us-east-1
    UsEast1,
    /// us-east-2
    UsEast2,
    /// us-west-1
    UsWest1,
    /// us-west-2
    UsWest2,
    /// ca-central-1
    CaCentral1,
    /// af-south-1
    AfSouth1,
    /// ap-east-1
    ApEast1,
    /// ap-south-1
    ApSouth1,
    /// ap-northeast-1
    ApNortheast1,
    /// ap-northeast-2
    ApNortheast2,
    /// ap-northeast-3
    ApNortheast3,
    /// ap-southeast-1
    ApSoutheast1,
    /// ap-southeast-2
    ApSoutheast2,
    /// cn-north-1
    CnNorth1,
    /// cn-northwest-1
    CnNorthwest1,
    /// eu-north-1
    EuNorth1,
    /// eu-central-1
    EuCentral1,
    /// eu-central-2
    EuCentral2,
    /// eu-west-1
    EuWest1,
    /// eu-west-2
    EuWest2,
    /// eu-west-3
    EuWest3,
    /// il-central-1
    IlCentral1,
    /// me-south-1
    MeSouth1,
    /// sa-east-1
    SaEast1,
}
macro_rules! into_region {
    (
        $(
            $variant:ident => $region:literal
        ),*
    ) => {
        impl S3StorageRegion {
            /// The AWS region code, as the SDK and the S3 API spell it.
            pub const fn region_code(self) -> &'static str {
                match self {
                    $(
                        S3StorageRegion::$variant => $region,
                    )*
                }
            }
        }
        impl From<S3StorageRegion> for aws_config::Region {
            fn from(value: S3StorageRegion) -> Self {
                aws_config::Region::from_static(value.region_code())
            }
        }
    };
}

into_region!(
    UsEast1 => "us-east-1",
    UsEast2 => "us-east-2",
    UsWest1 => "us-west-1",
    UsWest2 => "us-west-2",
    CaCentral1 => "ca-central-1",
    AfSouth1 => "af-south-1",
    ApEast1 => "ap-east-1",
    ApSouth1 => "ap-south-1",
    ApNortheast1 => "ap-northeast-1",
    ApNortheast2 => "ap-northeast-2",
    ApNortheast3 => "ap-northeast-3",
    ApSoutheast1 => "ap-southeast-1",
    ApSoutheast2 => "ap-southeast-2",
    CnNorth1 => "cn-north-1",
    CnNorthwest1 => "cn-northwest-1",
    EuNorth1 => "eu-north-1",
    EuCentral1 => "eu-central-1",
    EuCentral2 => "eu-central-2",
    EuWest1 => "eu-west-1",
    EuWest2 => "eu-west-2",
    EuWest3 => "eu-west-3",
    IlCentral1 => "il-central-1",
    MeSouth1 => "me-south-1",
    SaEast1 => "sa-east-1"
);
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CustomRegion {
    pub custom_region: Option<String>,
    pub endpoint: Url,
}
impl CustomRegion {
    /// The region name to sign requests with.
    ///
    /// SigV4 always needs *some* region in the credential scope even when the endpoint is a
    /// self-hosted MinIO that ignores it, so fall back to `us-east-1` when none was configured —
    /// that is what S3-compatible servers conventionally accept.
    pub fn region(&self) -> aws_config::Region {
        match &self.custom_region {
            Some(name) => aws_config::Region::new(name.clone()),
            None => aws_config::Region::from_static("us-east-1"),
        }
    }
}
