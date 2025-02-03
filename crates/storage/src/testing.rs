use std::{env::current_dir, path::PathBuf};

use nr_core::testing::logging::TestingLoggerConfig;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
pub mod tests;
use crate::{
    local::{LocalConfig, LocalStorage, LocalStorageFactory},
    s3::{regions::CustomRegion, S3Config, S3Credentials, S3StorageFactory},
    StaticStorageFactory, StorageConfig, StorageConfigInner, StorageTypeConfig,
};
pub mod storage;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingStorageConfig {
    pub logging: TestingLoggerConfig,
    pub storage_test_configs: Vec<StorageConfig>,
}

impl Default for TestingStorageConfig {
    fn default() -> Self {
        let mut storage_test_configs = Vec::new();
        storage_test_configs.push(LocalStorage::test_storage_config());
        storage_test_configs.push(S3Config::test_storage_config());
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
                endpoint: "http://localhost:9000".into(),
            }),
            credentials: S3Credentials::new_access_key("MY_ACCESS_KEY", "MY_SECRET_KEY"),
            path_style: true,
        }
    }
}

pub fn start_storage_test(storage_type: &str) -> anyhow::Result<Option<StorageConfig>> {
    let storage_configs = get_storage_configs()?;

    let storage_config = storage_configs
        .into_iter()
        .find(|config| config.storage_config.storage_type == storage_type);
    Ok(storage_config)
}
pub fn get_storage_configs() -> anyhow::Result<Vec<StorageConfig>> {
    let config = testing_config_file()?;
    config.logging.init();
    Ok(config.storage_test_configs)
}

fn testing_config_file() -> anyhow::Result<TestingStorageConfig> {
    let config_file =
        if let Ok(env) = std::env::var("STORAGE_TEST_CONFIG").map(PathBuf::from) {
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
