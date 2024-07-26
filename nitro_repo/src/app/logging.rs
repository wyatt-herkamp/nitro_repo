use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use opentelemetry::StringValue;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{new_exporter, WithExportConfig};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace, Resource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::{EnvFilter, Layer};

use super::config::{get_current_directory, Mode};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub logging_directory: PathBuf,
    pub tracing: Option<TracingConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TracingConfig {
    pub endpoint: String,
    pub service_name: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let logging_dir = get_current_directory().join("logs");
        Self {
            logging_directory: logging_dir,
            tracing: None,
        }
    }
}
impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            endpoint: "127.0.0.1:5959".to_owned(),
            service_name: "nitro_repo".to_string(),
        }
    }
}
impl LoggingConfig {
    pub fn init(&self, mode: Mode) -> anyhow::Result<()> {
        let base_filter = match mode {
            Mode::Debug => "debug,nitro_repo=trace",
            Mode::Release => "info",
        };
        let otel_filter = format!("{base_filter},nitro_repo=trace,sqlx=debug");

        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| base_filter.into());
        let file_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| base_filter.into());
        let otel_env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| otel_filter.into());

        let fmt_layer = tracing_subscriber::Layer::with_filter(
            tracing_subscriber::fmt::layer().pretty(),
            env_filter,
        );
        // Rolling File fmt_layer
        let file = {
            let file_appender =
                tracing_appender::rolling::hourly(self.logging_directory.clone(), "nitro_repo.log");
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_file(true)
                .with_level(true)
                .with_writer(file_appender)
                .with_filter(file_filter)
        };
        global::set_text_map_propagator(TraceContextPropagator::new());

        let registry = tracing_subscriber::registry().with(fmt_layer).with(file);

        if let Some(tracing) = self.tracing.clone() {
            println!("Loading Tracing {tracing:#?}");
            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(new_exporter().tonic().with_endpoint(&tracing.endpoint))
                .with_trace_config(trace::config().with_resource(Resource::new(vec![
                    KeyValue::new(
                        "service.name",
                        Into::<StringValue>::into(tracing.service_name.clone()),
                    ),
                ])))
                .install_batch(opentelemetry_sdk::runtime::Tokio)?;

            let otel_layer = tracing_subscriber::Layer::with_filter(
                tracing_opentelemetry::layer().with_tracer(tracer),
                otel_env_filter,
            );

            registry.with(otel_layer).init();
        } else {
            registry.init();
        }
        Ok(())
    }
}
