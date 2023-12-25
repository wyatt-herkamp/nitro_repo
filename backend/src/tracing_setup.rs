use opentelemetry::{global, KeyValue, StringValue};
use opentelemetry_otlp::{new_exporter, WithExportConfig};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace, Resource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::settings::tracing::TracingConfiguration;

const DEFAULT_FILTER: &'static str = "debug,h2=off";

pub fn setup(tracing: TracingConfiguration) -> anyhow::Result<()> {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or(DEFAULT_FILTER.into());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_line_number(false)
        .with_file(false);

    global::set_text_map_propagator(TraceContextPropagator::new());

    let registry = tracing_subscriber::registry().with(filter).with(fmt_layer);

    if let Some(tracing) = tracing.open_telemetry {
        println!("Setting up OpenTelemetry tracing #{tracing:?}");
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(new_exporter().tonic().with_endpoint(&tracing.endpoint))
            .with_trace_config(
                trace::config().with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    Into::<StringValue>::into(tracing.service_name.clone()),
                )])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        registry.with(otel_layer).init();
    } else {
        registry.init();
    }

    Ok(())
}
