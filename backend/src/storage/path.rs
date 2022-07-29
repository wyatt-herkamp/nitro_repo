use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct StoragePath {
    segments: Vec<String>,
}
impl StoragePath {
    pub fn new() -> Self {
        Self { segments: vec![] }
    }
    pub fn join<S: Into<String>>(&mut self, segment: S) {
        self.segments.push(segment.into());
    }
    pub fn join_new<S: Into<String>>(&self, s: S) -> Self {
        let mut path = self.clone();
        path.join(s);
        path
    }
    pub fn join_system<P: Into<PathBuf>>(mut self, path: P) -> PathBuf {
        let mut path: PathBuf = path.into();
        for x in self.segments {
            path = path.join(x.as_str())
        }
        path
    }
}
impl From<String> for StoragePath {
    fn from(v: String) -> Self {
        Self {
            segments: v.split("/").map(|s| s.to_string()).collect(),
        }
    }
}
impl FromStr for StoragePath {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            segments: s.split("/").map(|s| s.to_string()).collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SystemStorageFile {
    pub name: String,
    pub is_dir: bool,
}
