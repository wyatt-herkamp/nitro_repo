mod otel;
use std::path::PathBuf;

use ahash::{HashMap, HashMapExt};
pub use otel::*;
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::{
    filter::Targets,
    fmt::{
        format::{self, Format},
        time::SystemTime,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub loggers: HashMap<String, AppLogger>,
    pub metrics: Option<MetricsConfig>,
    pub levels: LoggingLevels,
}
impl Default for LoggingConfig {
    fn default() -> Self {
        let mut loggers = HashMap::new();
        loggers.insert("app".to_string(), AppLogger::Otel(OtelConfig::default()));
        loggers.insert(
            "console".to_string(),
            AppLogger::Console(ConsoleLogger::default()),
        );
        loggers.insert(
            "file".to_string(),
            AppLogger::RollingFile(RollingFileLogger::default()),
        );
        Self {
            loggers,
            metrics: Some(MetricsConfig::default()),
            levels: LoggingLevels::actual_default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum AppLogger {
    Otel(OtelConfig),
    Console(ConsoleLogger),
    RollingFile(RollingFileLogger),
}
pub trait AppLoggerType {
    fn get_levels_mut(&mut self) -> &mut LoggingLevels;
}
impl AppLoggerType for AppLogger {
    fn get_levels_mut(&mut self) -> &mut LoggingLevels {
        match self {
            AppLogger::Otel(config) => &mut config.levels,
            AppLogger::Console(config) => &mut config.levels,
            AppLogger::RollingFile(config) => &mut config.levels,
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
    pub fn actual_default() -> Self {
        let mut others = HashMap::new();
        others.insert("nitro_repo".to_string(), LevelSerde::Debug);
        others.insert("nr_core".to_string(), LevelSerde::Debug);
        others.insert("nr_storage".to_string(), LevelSerde::Debug);

        others.insert("h2".to_string(), LevelSerde::Warn);
        others.insert("tower".to_string(), LevelSerde::Warn);
        others.insert("tonic".to_string(), LevelSerde::Warn);
        others.insert("hyper_util".to_string(), LevelSerde::Warn);

        Self {
            default: LevelSerde::Info,
            others,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum LevelSerde {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl From<LevelSerde> for LevelFilter {
    fn from(level: LevelSerde) -> Self {
        match level {
            LevelSerde::Error => LevelFilter::ERROR,
            LevelSerde::Warn => LevelFilter::WARN,
            LevelSerde::Info => LevelFilter::INFO,
            LevelSerde::Debug => LevelFilter::DEBUG,
            LevelSerde::Trace => LevelFilter::TRACE,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardLoggerFmtRules {
    pub include_time: bool,
    pub include_level: bool,
    pub include_line_numbers: bool,
    pub include_file: bool,
    pub include_target: bool,
    pub ansi_color: bool,
    pub include_thread_ids: bool,
    pub include_thread_names: bool,
}
impl Default for StandardLoggerFmtRules {
    fn default() -> Self {
        Self {
            include_time: true,
            include_level: true,
            include_line_numbers: false,
            include_file: false,
            include_target: true,
            ansi_color: true,
            include_thread_ids: false,
            include_thread_names: false,
        }
    }
}
impl StandardLoggerFmtRules {
    pub fn layer_pretty<S>(
        &self,
    ) -> tracing_subscriber::fmt::Layer<S, format::Pretty, format::Format<format::Pretty, SystemTime>>
    {
        self.layer().pretty()
    }
    pub fn layer_compact<S>(
        &self,
    ) -> tracing_subscriber::fmt::Layer<S, format::DefaultFields, Format<format::Compact, SystemTime>>
    {
        self.layer().compact()
    }
    pub fn layer<S>(
        &self,
    ) -> tracing_subscriber::fmt::Layer<S, format::DefaultFields, Format<format::Full, SystemTime>>
    {
        tracing_subscriber::fmt::layer::<S>()
            .with_ansi(self.ansi_color)
            .with_target(self.include_target)
            .with_line_number(self.include_line_numbers)
            .with_file(self.include_file)
            .with_level(self.include_level)
            .with_thread_ids(self.include_thread_ids)
            .with_thread_names(self.include_thread_names)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsoleLogger {
    pub pretty: bool,
    #[serde(flatten)]
    pub rules: StandardLoggerFmtRules,
    pub levels: LoggingLevels,
}
impl AppLoggerType for ConsoleLogger {
    fn get_levels_mut(&mut self) -> &mut LoggingLevels {
        &mut self.levels
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingFileLogger {
    pub path: PathBuf,
    pub file_prefix: String,
    pub levels: LoggingLevels,

    pub interval: RollingInterval,
    #[serde(flatten)]
    pub rules: StandardLoggerFmtRules,
}
impl AppLoggerType for RollingFileLogger {
    fn get_levels_mut(&mut self) -> &mut LoggingLevels {
        &mut self.levels
    }
}
impl Default for RollingFileLogger {
    fn default() -> Self {
        Self {
            path: PathBuf::from("logs/app.log"),
            file_prefix: "thd-helper.log".to_string(),
            levels: LoggingLevels::default(),
            interval: RollingInterval::Daily,
            rules: StandardLoggerFmtRules::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollingInterval {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl From<RollingInterval> for Rotation {
    fn from(value: RollingInterval) -> Self {
        match value {
            RollingInterval::Minutely => Rotation::MINUTELY,
            RollingInterval::Hourly => Rotation::HOURLY,
            RollingInterval::Daily => Rotation::DAILY,
            RollingInterval::Never => Rotation::NEVER,
        }
    }
}
