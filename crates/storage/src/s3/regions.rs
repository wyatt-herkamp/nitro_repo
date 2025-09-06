use serde::{Deserialize, Serialize};
use strum::EnumIter;
use tux_io_s3::types::region::{CustomRegion as TuxIoCustomRegion, OfficialRegion};
use url::Url;
use utoipa::ToSchema;

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
            $variant:ident => $region:ident
        ),*
    ) => {
        impl From<S3StorageRegion> for OfficialRegion{
            fn from(value: S3StorageRegion) -> Self {
                match value {
                    $(
                        S3StorageRegion::$variant => OfficialRegion::$region,
                    )*
                }
            }
        }
    };
}

into_region!(
    UsEast1 => UsEast1,
    UsEast2 => UsEast2,
    UsWest1 => UsWest1,
    UsWest2 => UsWest2,
    CaCentral1 => CaCentral1,
    AfSouth1 => AfSouth1,
    ApEast1 => ApEast1,
    ApSouth1 => ApSouth1,
    ApNortheast1 => ApNortheast1,
    ApNortheast2 => ApNortheast2,
    ApNortheast3 => ApNortheast3,
    ApSoutheast1 => ApSoutheast1,
    ApSoutheast2 => ApSoutheast2,
    CnNorth1 => CnNorth1,
    CnNorthwest1 => CnNorthwest1,
    EuNorth1 => EuNorth1,
    EuCentral1 => EuCentral1,
    EuCentral2 => EuCentral2,
    EuWest1 => EuWest1,
    EuWest2 => EuWest2,
    EuWest3 => EuWest3,
    IlCentral1 => IlCentral1,
    MeSouth1 => MeSouth1,
    SaEast1 => SaEast1
);
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CustomRegion {
    pub custom_region: Option<String>,
    pub endpoint: Url,
}
impl From<CustomRegion> for TuxIoCustomRegion {
    fn from(value: CustomRegion) -> Self {
        TuxIoCustomRegion {
            name: value.custom_region,
            endpoint: value.endpoint,
        }
    }
}
