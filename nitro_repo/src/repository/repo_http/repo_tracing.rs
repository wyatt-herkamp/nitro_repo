use std::{error::Error, mem, sync::Arc};

use nr_core::storage::StoragePath;
use nr_storage::Storage;
use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    KeyValue,
};
use parking_lot::Mutex;
use tracing::{event, field::Empty, info_span, trace, Level, Span};

use crate::repository::Repository;

use super::DynRepository;
#[derive(Debug, Clone)]
pub struct RepositoryMetricsMeter {
    meter: Meter,
    project_access_bytes: Histogram<u64>,
    project_number_of_versions: UpDownCounter<i64>,
    project_write_bytes: Histogram<u64>,
}
impl Default for RepositoryMetricsMeter {
    fn default() -> Self {
        let meter = global::meter("nitro_repo::repository::metrics");
        Self {
            project_access_bytes: meter.u64_histogram("nr.project.access.bytes").build(),
            project_number_of_versions: meter.i64_up_down_counter("nr.project.versions").build(),
            project_write_bytes: meter.u64_histogram("nr.project.write.bytes").build(),

            meter: meter,
        }
    }
}
#[derive(Debug, Default)]
pub struct RepoMetricsSet {
    pub project_access_bytes: Option<u64>,
    pub project_write_bytes: Option<u64>,
}
#[derive(Debug)]
pub struct RepositoryMetrics {
    metric_attributes: Mutex<Vec<KeyValue>>,
    pub meter: RepositoryMetricsMeter,
    metrics: Mutex<RepoMetricsSet>,
    span: Span,
}
impl Drop for RepositoryMetrics {
    fn drop(&mut self) {
        let _guard = self.span.enter();
        let metrics = {
            let mut metrics = self.metrics.lock();
            mem::replace(&mut *metrics, Default::default())
        };
        let attributes = {
            let mut attributes = self.metric_attributes.lock();
            mem::replace(&mut *attributes, Default::default())
        };
        if let Some(bytes) = metrics.project_access_bytes {
            event!(Level::TRACE, ?bytes, "Recording project access bytes");
            self.meter.project_access_bytes.record(bytes, &attributes);
        }
        if let Some(bytes) = metrics.project_write_bytes {
            event!(Level::TRACE, ?bytes, "Recording project write bytes");
            self.meter.project_write_bytes.record(bytes, &attributes);
        }
    }
}
impl RepositoryMetrics {
    pub fn new(meter: RepositoryMetricsMeter, span: Span) -> Self {
        Self {
            metric_attributes: Default::default(),
            meter,
            metrics: Default::default(),
            span,
        }
    }
}

impl RepositoryMetrics {
    pub fn add_attribute(&self, key: &'static str, value: impl Into<opentelemetry::Value>) {
        let value = value.into();
        let mut metric_data = self.metric_attributes.lock();
        metric_data.push(KeyValue::new(key, value));
    }
    pub fn project_access_bytes(&self, bytes: u64) {
        self.metrics.lock().project_access_bytes = Some(bytes);
    }
    pub fn project_write_bytes(&self, bytes: u64) {
        self.metrics.lock().project_write_bytes = Some(bytes);
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryRequestTracing {
    pub span: tracing::Span,
    pub metrics: Arc<RepositoryMetrics>,
}
impl RepositoryRequestTracing {
    pub fn new(
        repository: &DynRepository,
        parent_span: &Span,
        metrics: RepositoryMetricsMeter,
    ) -> Self {
        let span = info_span!(
            target: "nitro_repo::repository::requests",
            parent: parent_span,
            "Repository Request",
            "repository.type" = Empty,
            "repository.full_type" = Empty,
            "repository.name" =Empty,
            "repository.id" = Empty,
            "project.scope" = Empty,
            "project.name" = Empty,
            "project.key" = Empty,
            "project.version" = Empty,
            "storage.id" = Empty,
            "storage.type" = Empty,
            "storage.name" =Empty,
            "storage.path" = Empty,
            otel.name = format!("{type} {name}", type = repository.full_type(), name = repository.name()),
            otel.status_code = Empty,
            exception.message = Empty,
        );

        let result = Self {
            span: span.clone(),
            metrics: Arc::new(RepositoryMetrics::new(metrics, span.clone())),
        };
        let storage = repository.get_storage();
        let storage_config = storage.storage_config();
        result.push_metric_and_span("repository.type", repository.get_type());
        result.push_metric_and_span("repository.full_type", repository.full_type());
        result.push_metric_and_span("repository.name", &repository.name());
        result.push_metric_and_span("repository.id", &repository.id().to_string());
        result.push_metric_and_span(
            "storage.id",
            &storage_config.storage_config.storage_id.to_string(),
        );
        result.push_metric_and_span("storage.type", &storage_config.storage_config.storage_type);
        result.push_metric_and_span("storage.name", &storage_config.storage_config.storage_name);

        result
    }
    pub(super) fn path(&self, path: &StoragePath) {
        let path = path.to_string();
        self.push_metric_and_span("storage.path", &path);
    }
    pub fn set_project(
        &self,
        project_scope: String,
        project_name: String,
        project_key: String,
        project_version: Option<String>,
    ) {
        self.push_metric_and_span("project.scope", &project_scope);
        self.push_metric_and_span("project.name", &project_name);
        self.push_metric_and_span("project.key", &project_key);
        if let Some(project_version) = project_version {
            self.push_metric_and_span("project.version", &project_version);
        }
    }
    pub fn add_metric_attribute(&self, key: &'static str, value: impl Into<opentelemetry::Value>) {
        let value = value.into();
        self.metrics.add_attribute(key, value);
    }

    fn push_metric_and_span(&self, key: &'static str, value: &str) {
        self.span.record(key, &value);
        self.metrics.add_attribute(key, value.to_string());
    }
    pub(super) fn error(&self, error: impl Error) {
        self.span.record("exception.message", &error.to_string());
        self.span.record("otel.status_code", "ERROR");
    }
    pub(super) fn ok(&self) {
        self.span.record("otel.status_code", "OK");
    }
}
