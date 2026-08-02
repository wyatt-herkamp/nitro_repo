use std::path::PathBuf;

use ahash::{HashMap, HashMapExt};
use tracing::{debug, instrument};

#[instrument]
pub fn find_file(dir: PathBuf, file_name: &str) -> Option<PathBuf> {
    let env_file = dir.join(file_name);
    debug!("Checking for file: {:?}", env_file);
    if env_file.exists() {
        return Some(env_file);
    }
    let parent = dir.parent()?;
    debug!("Checking parent: {:?}", parent);
    find_file(parent.to_path_buf(), file_name)
}

#[derive(Debug)]
pub struct EnvFile {
    /// `None` when there was no file and the values come from the environment alone.
    pub file: Option<PathBuf>,
    pub key_values: HashMap<String, String>,
}

impl EnvFile {
    /// Loads a `key=value` file, falling back to the process environment.
    ///
    /// A missing file is not an error. `nr_tests.env` is gitignored — it holds whatever local
    /// database a developer happens to use — so it does not exist in CI, and this used to fail with
    /// "File not found" before [`Self::get`] ever got the chance to read `DATABASE_URL` from the
    /// environment. Every test built on `TestCore` was therefore unrunnable anywhere but a machine
    /// that had the file.
    pub fn load(file_name: &str) -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let Some(file) = find_file(current_dir, file_name) else {
            debug!("No {file_name}; using the environment only");
            return Ok(Self {
                file: None,
                key_values: HashMap::new(),
            });
        };

        let file_contents = std::fs::read_to_string(&file)?;
        let mut key_values = HashMap::new();
        for line in file_contents.lines() {
            let line = line.trim();
            // Blank lines and comments used to reach `split_once(..).unwrap()` and panic.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                debug!("Ignoring a line with no `=`: {line}");
                continue;
            };
            key_values.insert(key.trim().to_owned(), value.trim().to_owned());
        }

        Ok(Self {
            file: Some(file),
            key_values,
        })
    }

    /// The environment wins over the file, so CI can override without editing anything.
    pub fn get(&self, key: &str) -> Option<String> {
        if let Ok(value) = std::env::var(key) {
            return Some(value);
        }
        self.key_values.get(key).map(|value| value.to_owned())
    }
}
