use ahash::{HashMap, HashMapExt};
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum LevelSerde {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Off,
}
impl From<LevelSerde> for LevelFilter {
    fn from(level: LevelSerde) -> Self {
        match level {
            LevelSerde::Error => LevelFilter::ERROR,
            LevelSerde::Warn => LevelFilter::WARN,
            LevelSerde::Info => LevelFilter::INFO,
            LevelSerde::Debug => LevelFilter::DEBUG,
            LevelSerde::Trace => LevelFilter::TRACE,
            LevelSerde::Off => LevelFilter::OFF,
        }
    }
}
impl From<LevelFilter> for LevelSerde {
    fn from(level: LevelFilter) -> Self {
        match level {
            LevelFilter::ERROR => LevelSerde::Error,
            LevelFilter::WARN => LevelSerde::Warn,
            LevelFilter::INFO => LevelSerde::Info,
            LevelFilter::DEBUG => LevelSerde::Debug,
            LevelFilter::TRACE => LevelSerde::Trace,
            LevelFilter::OFF => LevelSerde::Off,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingLevels {
    pub default: LevelSerde,
    pub others: HashMap<String, LevelSerde>,
}
impl From<LoggingLevels> for Targets {
    fn from(targets: LoggingLevels) -> Self {
        let mut builder = tracing_subscriber::filter::Targets::new();

        builder = builder.with_default(targets.default);
        for (name, level) in targets.others {
            builder = builder.with_target(name, level);
        }
        builder
    }
}

impl Default for LoggingLevels {
    fn default() -> Self {
        Self {
            default: LevelSerde::Info,
            others: Default::default(),
        }
    }
}

impl LoggingLevels {
    /// Inherit the levels from another logging levels.
    ///
    /// This will check if Self contains a key from other if not it will insert it.
    pub fn inherit_from(&mut self, other: &LoggingLevels) {
        for (k, v) in other.others.iter() {
            if !self.others.contains_key(k) {
                self.others.insert(k.clone(), v.clone());
            }
        }
    }
}
