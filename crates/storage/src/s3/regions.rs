use s3::Region;
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use utoipa::ToSchema;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ToSchema, EnumIter)]
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
    /// Digital Ocean nyc3
    DoNyc3,
    /// Digital Ocean ams3
    DoAms3,
    /// Digital Ocean sgp1
    DoSgp1,
    /// Digital Ocean fra1
    DoFra1,
    /// Yandex Object Storage
    Yandex,
    /// Wasabi us-east-1
    WaUsEast1,
    /// Wasabi us-east-2
    WaUsEast2,
    /// Wasabi us-central-1
    WaUsCentral1,
    /// Wasabi us-west-1
    WaUsWest1,
    /// Wasabi ca-central-1
    WaCaCentral1,
    /// Wasabi eu-central-1
    WaEuCentral1,
    /// Wasabi eu-central-2
    WaEuCentral2,
    /// Wasabi eu-west-1
    WaEuWest1,
    /// Wasabi eu-west-2
    WaEuWest2,
    /// Wasabi ap-northeast-1
    WaApNortheast1,
    /// Wasabi ap-northeast-2
    WaApNortheast2,
    /// Wasabi ap-southeast-1
    WaApSoutheast1,
    /// Wasabi ap-southeast-2
    WaApSoutheast2,
}
macro_rules! into_region {
    (
        $(
            $variant:ident => $region:ident
        ),*
    ) => {
        impl From<S3StorageRegion> for Region{
            fn from(value: S3StorageRegion) -> Self {
                match value {
                    $(
                        S3StorageRegion::$variant => Region::$region,
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
    SaEast1 => SaEast1,
    DoNyc3 => DoNyc3,
    DoAms3 => DoAms3,
    DoSgp1 => DoSgp1,
    DoFra1 => DoFra1,
    Yandex => Yandex,
    WaUsEast1 => WaUsEast1,
    WaUsEast2 => WaUsEast2,
    WaUsCentral1 => WaUsCentral1,
    WaUsWest1 => WaUsWest1,
    WaCaCentral1 => WaCaCentral1,
    WaEuCentral1 => WaEuCentral1,
    WaEuCentral2 => WaEuCentral2,
    WaEuWest1 => WaEuWest1,
    WaEuWest2 => WaEuWest2,
    WaApNortheast1 => WaApNortheast1,
    WaApNortheast2 => WaApNortheast2,
    WaApSoutheast1 => WaApSoutheast1,
    WaApSoutheast2 => WaApSoutheast2


);
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CustomRegion {
    pub custom_region: Option<String>,
    pub endpoint: String,
}
impl From<CustomRegion> for Region {
    fn from(value: CustomRegion) -> Self {
        Region::Custom {
            region: value
                .custom_region
                .unwrap_or_else(|| "custom-region".to_string()),
            endpoint: value.endpoint,
        }
    }
}
