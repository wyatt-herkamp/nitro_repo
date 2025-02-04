use std::path::PathBuf;
use utoipa::OpenApi;

use std::fs::{File, create_dir_all};

use crate::app::{REPOSITORY_CONFIG_TYPES, REPOSITORY_TYPES};

pub fn export_repository_configs(path: PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        create_dir_all(&path)?;
    }
    if path.is_file() {
        return Err(anyhow::anyhow!("Path is a file, expected a directory"));
    }
    for config_type in REPOSITORY_CONFIG_TYPES {
        {
            let Some(schema) = config_type.schema() else {
                println!(
                    "Skipping Schema for {} Error: no schema",
                    config_type.get_type()
                );
                continue;
            };
            let file_path = path.join(format!("{}.schema.json", config_type.get_type()));
            let file = File::create(&file_path)?;
            serde_json::to_writer_pretty(file, &schema)?;
            println!(
                "Exported Schema: {} to {}",
                config_type.get_type(),
                file_path.display()
            );
        }
        {
            let file_path = path.join(format!("{}.description.json", config_type.get_type()));
            let file = File::create(&file_path)?;
            serde_json::to_writer_pretty(file, &config_type.get_description())?;
            println!(
                "Exported Default: {} to {}",
                config_type.get_type(),
                file_path.display()
            );
        }
    }
    Ok(())
}
pub fn export_repository_types(path: PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        create_dir_all(&path)?;
    }
    if path.is_file() {
        return Err(anyhow::anyhow!("Path is a file, expected a directory"));
    }

    for repository_type in REPOSITORY_TYPES {
        let file_path = path.join(format!("{}.json", repository_type.get_type()));
        let file = File::create(&file_path)?;
        serde_json::to_writer_pretty(file, &repository_type.get_description())?;
        println!(
            "Exported: {} to {}",
            repository_type.get_type(),
            file_path.display()
        );
    }
    Ok(())
}
pub fn export_openapi(path: PathBuf) -> anyhow::Result<()> {
    let path = if path.is_dir() {
        println!("Path is a directory, using default name");
        path.join("nitro-repo-open-api.json")
    } else {
        path
    };

    let open_api = crate::app::open_api::ApiDoc::openapi();
    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, &open_api)?;
    println!("Exported OpenAPI to {}", path.display());
    Ok(())
}
