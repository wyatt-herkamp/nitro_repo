use super::Repository;
#[derive(Debug, Clone)]
pub enum DynRepository {
    Maven(super::maven::MavenRepository),
}
impl Repository for DynRepository {
    fn get_storage(&self) -> nr_storage::DynStorage {
        match self {
            DynRepository::Maven(maven) => maven.get_storage(),
        }
    }

    fn get_type(&self) -> &'static str {
        match self {
            DynRepository::Maven(maven) => maven.get_type(),
        }
    }

    fn config_types(&self) -> Vec<String> {
        match self {
            DynRepository::Maven(maven) => maven.config_types(),
        }
    }
}
