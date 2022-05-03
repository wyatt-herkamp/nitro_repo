use crate::repository::data::{RepositoryConfig, RepositoryType};
use crate::repository::error::RepositoryError;
use crate::repository::error::RepositoryError::InternalError;

use crate::repository::maven::models::MavenSettings;

use crate::repository::npm::models::NPMSettings;

use crate::storage::models::Storage;

pub enum NitroRepository {
    Maven(RepositoryConfig<MavenSettings>),
    NPM(RepositoryConfig<NPMSettings>),
}

impl NitroRepository {
    pub async fn load(
        storage: &Storage,
        name: &str,
    ) -> Result<Option<NitroRepository>, RepositoryError> {
        let repository_value = storage.get_repository_value(name).await?;
        if repository_value.is_none() {
            return Ok(None);
        }
        let repository_value = repository_value.unwrap();
        return match &repository_value.repository_type {
            RepositoryType::Maven => {
                let main = storage.get_repository::<MavenSettings>(name).await?;
                if main.is_none() {
                    return Err(InternalError(
                        "Repository Registered but not found".to_string(),
                    ));
                }
                Ok(Some(NitroRepository::Maven(main.unwrap())))
            }
            RepositoryType::NPM => {
                let main = storage.get_repository::<NPMSettings>(name).await?;
                if main.is_none() {
                    return Err(InternalError(
                        "Repository Registered but not found".to_string(),
                    ));
                }

                Ok(Some(NitroRepository::NPM(main.unwrap())))
            }
        };
    }
}
