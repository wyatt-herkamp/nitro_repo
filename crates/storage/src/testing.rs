use std::{env::current_dir, path::PathBuf};

use nr_core::testing::logging::TestingLoggerConfig;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
pub mod tests;
use crate::{
    StaticStorageFactory, StorageConfig, StorageConfigInner, StorageTypeConfig,
    fs_v2::{Compression, FileSystemV2Config, FileSystemV2Factory, FileSystemV2Storage},
    local::{LocalConfig, LocalStorage, LocalStorageFactory},
    s3::{S3Config, S3Credentials, S3StorageFactory, regions::CustomRegion},
};
pub mod storage;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingStorageConfig {
    pub logging: TestingLoggerConfig,
    pub storage_test_configs: Vec<StorageConfig>,
}

impl Default for TestingStorageConfig {
    fn default() -> Self {
        let storage_test_configs = vec![
            LocalStorage::test_storage_config(),
            S3Config::test_storage_config(),
            FileSystemV2Storage::test_storage_config(),
        ];
        Self {
            logging: TestingLoggerConfig::default(),
            storage_test_configs,
        }
    }
}

pub trait TestingStorageType {
    type ConfigType;
    type Factory: StaticStorageFactory + Default;

    fn test_config() -> Self::ConfigType;

    fn test_storage_config() -> StorageConfig
    where
        Self::ConfigType: Into<StorageTypeConfig>,
    {
        StorageConfig {
            storage_config: StorageConfigInner {
                storage_name: "test".into(),
                storage_id: Uuid::new_v4(),
                storage_type: Self::Factory::storage_type_name().to_owned(),
                created_at: Default::default(),
            },
            type_config: Self::test_config().into(),
        }
    }
}

impl TestingStorageType for LocalStorage {
    type ConfigType = LocalConfig;
    type Factory = LocalStorageFactory;
    fn test_config() -> Self::ConfigType {
        LocalConfig {
            path: testing_storage_directory()
                .unwrap()
                .join("local_storage_test"),
        }
    }
}
impl TestingStorageType for S3Config {
    type ConfigType = S3Config;
    type Factory = S3StorageFactory;
    fn test_config() -> S3Config {
        S3Config {
            bucket_name: "test-bucket".into(),
            region: None,
            custom_region: Some(CustomRegion {
                custom_region: Some("minio-instance".to_owned()),
                endpoint: "http://localhost:9000".parse().unwrap(),
            }),
            credentials: S3Credentials::new_access_key("nitro_repo", "nitro_repo_password"),
            path_style: true,
        }
    }
}

impl TestingStorageType for FileSystemV2Storage {
    type ConfigType = FileSystemV2Config;
    type Factory = FileSystemV2Factory;
    fn test_config() -> Self::ConfigType {
        FileSystemV2Config {
            path: testing_storage_directory().unwrap().join("fs_v2_test"),
            compression: Compression::None,
            sync: false,
        }
    }
}

/// Resolves the config for a backend's test run, or `None` when that backend cannot be tested here.
///
/// Two distinct reasons to skip, which used to be conflated:
///
/// - **No config.** Nothing to test against; skip quietly.
/// - **Config present but the service is unreachable.** A default config is written automatically
///   on first run and names a MinIO on `localhost:9000`, so a developer with no MinIO running had
///   no way *not* to run the S3 test — and it failed the whole suite on "connection refused".
///   That is an environment gap, not a defect, so it skips with a warning.
///
/// Set `STORAGE_TESTS_REQUIRE_ALL=1` to turn the second case back into a failure. CI sets it once
/// the services are up, so a genuinely broken backend cannot hide behind a skip.
pub async fn start_storage_test(storage_type: &str) -> anyhow::Result<Option<StorageConfig>> {
    let storage_configs = get_storage_configs()?;

    let Some(storage_config) = storage_configs
        .into_iter()
        .find(|config| config.storage_config.storage_type == storage_type)
    else {
        info!(storage_type, "No test config for this storage type");
        return Ok(None);
    };

    if let Err(err) = probe_storage_config(&storage_config).await {
        if require_all_storage_tests() {
            return Err(err.context(format!(
                "{storage_type} is unreachable and STORAGE_TESTS_REQUIRE_ALL is set"
            )));
        }
        warn!(
            storage_type,
            ?err,
            "Storage backend is unreachable; skipping. Set STORAGE_TESTS_REQUIRE_ALL=1 to fail instead."
        );
        return Ok(None);
    }

    Ok(Some(storage_config))
}

fn require_all_storage_tests() -> bool {
    std::env::var("STORAGE_TESTS_REQUIRE_ALL")
        .map(|value| !matches!(value.as_str(), "" | "0" | "false"))
        .unwrap_or(false)
}

/// Checks a backend is actually usable before its test suite commits to running.
async fn probe_storage_config(config: &StorageConfig) -> anyhow::Result<()> {
    for factory in crate::STORAGE_FACTORIES {
        if factory.storage_name() == config.storage_config.storage_type {
            factory
                .test_storage_config(config.type_config.clone())
                .await?;
            return Ok(());
        }
    }
    anyhow::bail!(
        "No storage factory named `{}`",
        config.storage_config.storage_type
    )
}
pub fn get_storage_configs() -> anyhow::Result<Vec<StorageConfig>> {
    let config = testing_config_file()?;
    config.logging.init();
    Ok(config.storage_test_configs)
}

fn testing_config_file() -> anyhow::Result<TestingStorageConfig> {
    let config_file = if let Ok(env) = std::env::var("STORAGE_TEST_CONFIG").map(PathBuf::from) {
        env
    } else {
        testing_storage_directory()?.join("storage_testing_config.toml")
    };
    if !config_file.exists() {
        let config = TestingStorageConfig::default();
        let toml = toml::to_string(&config)?;
        std::fs::write(&config_file, toml)?;
        return Ok(config);
    }

    let config = toml::from_str(&std::fs::read_to_string(&config_file)?)?;
    Ok(config)
}

fn testing_storage_directory() -> anyhow::Result<PathBuf> {
    let env = std::env::var("STORAGE_TEST_DIRECTORY")
        .map(PathBuf::from)
        .ok();
    let dir = if let Some(env) = env {
        env
    } else {
        current_dir()
            .expect("Unable to get Working Dir")
            .join("storage_tests")
    };
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    if dir.is_file() {
        anyhow::bail!("Storage Test Directory is a file");
    }
    Ok(dir)
}

#[test]
fn test_load_config() -> anyhow::Result<()> {
    let configs = get_storage_configs()?;
    for config in configs {
        info!("{:?}", config);
    }
    Ok(())
}
