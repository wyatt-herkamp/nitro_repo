use std::{error::Error, sync::Arc};

use nr_core::storage::StoragePath;
use nr_storage::Storage;
use opentelemetry::{
    global,
    metrics::{Counter, Meter},
    KeyValue,
};
use parking_lot::Mutex;
use tracing::{field::Empty, info_span, Span};

use crate::repository::Repository;

use super::DynRepository;
#[derive(Debug, Clone)]
pub struct RepositoryMetricsMeter {
    meter: Meter,
}
impl Default for RepositoryMetricsMeter {
    fn default() -> Self {
        let meter = global::meter("nitro_repo::repository::metrics");
        Self { meter: meter }
    }
}

#[derive(Debug)]
pub struct RepositoryMetrics {
    metric_attributes: Mutex<Vec<KeyValue>>,
    pub meter: RepositoryMetricsMeter,
}
impl RepositoryMetrics {
    pub fn new(meter: RepositoryMetricsMeter) -> Self {
        Self {
            metric_attributes: Default::default(),
            meter,
        }
    }
}

impl RepositoryMetrics {
    pub fn add_attribute(&self, key: &'static str, value: impl Into<opentelemetry::Value>) {
        let value = value.into();
        let mut metric_data = self.metric_attributes.lock();
        metric_data.push(KeyValue::new(key, value));
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryRequestTracing {
    pub span: tracing::Span,
    metrics: Arc<RepositoryMetrics>,
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
            "repository.project.scope" = Empty,
            "repository.project.name" = Empty,
            "repository.project.key" = Empty,
            "repository.project.version" = Empty,
            "storage.id" = Empty,
            "storage.type" = Empty,
            "storage.name" =Empty,
            "storage.path" = Empty,
            otel.name = format!("{type} {name}", type = repository.full_type(), name = repository.name()),
            otel.status_code = Empty,
            exception.message = Empty,
        );

        let result = Self {
            span,
            metrics: Arc::new(RepositoryMetrics::new(metrics)),
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
        project_scope: impl Into<String>,
        project_name: impl Into<String>,
        project_key: impl Into<String>,
        project_version: impl Into<String>,
    ) {
        let project_name = project_name.into();
        let project_key = project_key.into();
        let project_version = project_version.into();
        self.push_metric_and_span("repository.project.scope", &project_scope.into());
        self.push_metric_and_span("repository.project.name", &project_name);
        self.push_metric_and_span("repository.project.key", &project_key);
        self.push_metric_and_span("repository.project.version", &project_version);
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
