use std::{collections::HashMap, path::PathBuf};

use tracing::{debug, info, instrument};
#[instrument]
pub fn find_file(dir: PathBuf, file_name: &str) -> Option<PathBuf> {
    let env_file = dir.join(file_name);
    info!("Checking for file: {:?}", env_file);
    if env_file.exists() {
        return Some(env_file);
    }
    let parent = dir.parent()?;
    debug!("Checking parent: {:?}", parent);
    find_file(parent.to_path_buf(), file_name)
}
#[derive(Debug)]
pub struct EnvFile {
    pub file: PathBuf,
    pub key_values: HashMap<String, String>,
}
impl EnvFile {
    pub fn load(file_name: &str) -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let file =
            find_file(current_dir, file_name).ok_or_else(|| anyhow::anyhow!("File not found"))?;
        let file_contents = std::fs::read_to_string(&file)?;
        let mut key_values = HashMap::new();
        for line in file_contents.lines() {
            let (key, value) = line.split_once('=').unwrap();
            
            
            key_values.insert(key.to_string(), value.to_string());
        }
        Ok(Self { file, key_values })
    }
    pub fn get(&self, key: &str) -> Option<String> {
        if let Ok(key) = std::env::var(key) {
            return Some(key);
        }
        self.key_values.get(key).map(|s| s.to_owned())
    }
}
