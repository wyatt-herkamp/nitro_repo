use std::path::PathBuf;

use ahash::{HashMap, HashMapExt};
use opentelemetry::trace::TracerProvider;
use opentelemetry::StringValue;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{new_exporter, WithExportConfig};
use opentelemetry_sdk::trace::{Config as SDKTraceConfig, Tracer};
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::{EnvFilter, Layer};
pub mod request_tracing;
use super::config::{get_current_directory, Mode};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub logging_directory: PathBuf,
    pub tracing: Option<TracingConfig>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TracingConfig {
    pub endpoint: String,
    /// Tracing Config Resource Values.
    ///
    /// ```toml
    /// "service.name" = "nitro_repo"
    /// "service.version" = "0.1.0"
    /// "service.environment" = "development"
    /// ```
    pub trace_config: HashMap<String, String>,
}
impl TracingConfig {
    fn tracer(mut self) -> anyhow::Result<Tracer> {
        println!("Loading Tracing {self:#?}");

        if !self.trace_config.contains_key("service.name") {
            self.trace_config
                .insert("service.name".to_owned(), "nitro_repo".to_owned());
        }
        let resources: Vec<KeyValue> = self
            .trace_config
            .into_iter()
            .map(|(k, v)| KeyValue::new(k, Into::<StringValue>::into(v.clone())))
            .collect();
        let trace_config = SDKTraceConfig::default().with_resource(Resource::new(resources));

        let exporter = new_exporter().tonic().with_endpoint(&self.endpoint);
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(trace_config)
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        Ok(provider.tracer("tracing-otel-subscriber"))
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        let mut trace_config = HashMap::new();
        trace_config.insert("service.name".to_owned(), "nitro_repo".to_owned());
        Self {
            endpoint: "127.0.0.1:5959".to_owned(),
            trace_config,
        }
    }
}
impl LoggingConfig {
    pub fn init(&self, mode: Mode) -> anyhow::Result<()> {
        let base_filter = match mode {
            Mode::Debug => {
                "debug,nitro_repo=trace,nr_storage=trace,nr_core=trace,h2=warn,tower=warn,hyper_util=warn,lettre=trace"
            }
            Mode::Release => "info",
        };
        let otel_filter = "debug,nitro_repo=trace,nr_storage=trace,nr_core=trace,tower=warn,hyper_util=warn".to_string();

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
            let tracer = tracing.tracer()?;
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
