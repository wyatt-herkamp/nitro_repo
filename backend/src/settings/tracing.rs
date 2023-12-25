use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TracingConfiguration {
    pub open_telemetry: Option<OpenTelemetryTracing>,
}
impl Default for TracingConfiguration {
    fn default() -> Self {
        Self {
            open_telemetry: None,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenTelemetryTracing {
    /// The OpenTelemetry endpoint to send traces to.
    pub endpoint: String,
    /// The service name to use for traces.
    pub service_name: String,
}
impl Default for OpenTelemetryTracing {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:14268".to_string(),
            service_name: env!("CARGO_PKG_NAME").to_string(),
        }
    }
}
