use std::sync::Once;

use ahash::{HashMap, HashMapExt};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::logging::{LevelSerde, LoggingLevels};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingLoggerConfig {
    pub levels: LoggingLevels,
}
impl TestingLoggerConfig {
    pub fn init(self) {
        static ONCE: Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let targets: Targets = self.levels.into();
            let stdout_log = tracing_subscriber::fmt::layer()
                .pretty()
                .without_time()
                .with_thread_ids(false)
                .with_thread_names(false);
            tracing_subscriber::registry()
                .with(stdout_log.with_filter(targets))
                .init();
            info!("Logger initialized");
        });
        trace!("This is a trace message");
        debug!("This is a debug message");
        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");
    }
}
impl Default for TestingLoggerConfig {
    fn default() -> Self {
        let mut others = HashMap::new();
        others.insert("nitro_repo".to_string(), LevelSerde::Debug);
        others.insert("nr_core".to_string(), LevelSerde::Debug);
        others.insert("nr_storage".to_string(), LevelSerde::Debug);

        others.insert("h2".to_string(), LevelSerde::Warn);
        others.insert("tower".to_string(), LevelSerde::Warn);
        others.insert("tonic".to_string(), LevelSerde::Warn);
        others.insert("hyper_util".to_string(), LevelSerde::Warn);
        Self {
            levels: LoggingLevels {
                default: LevelSerde::Debug,
                others: others,
            },
        }
    }
}
