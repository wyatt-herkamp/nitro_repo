use std::path;

use nr_core::storage::StoragePath;
use tokio::io::AsyncReadExt;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{FileContent, FileType, Storage, StorageError, StorageFile};

use super::{storage::TestingStorage, TestingStorageType};
pub async fn full_test<ST: Storage>(storage: TestingStorage<ST>) -> anyhow::Result<()> {
    write_then_read(&storage).await?;
    write_multiple_then_list(&storage).await?;
     should_conflict(&storage).await?;
    storage.unload().await?;
    Ok(())
}

pub async fn write_then_read<ST: Storage>(storage: &TestingStorage<ST>) -> anyhow::Result<()> {
    let repository = Uuid::new_v4();
    let path = StoragePath::from("test.txt");
    let content = FileContent::from("Hello, World!");

    let (_, _) = storage
        .save_file(repository, content.clone(), &path)
        .await?;
    let expected: Vec<u8> = content.try_into()?;

    let read_content = storage.open_file(repository, &path).await?;

    assert!(read_content.is_some(), "File not found");
    let read_content = read_content.unwrap();
    assert!(read_content.is_file(), "File is not a file");

    let StorageFile::File { meta, content } = read_content else {
        panic!("File is not a file");
    };
    let content = content
        .read_to_vec(meta.file_type.file_size as usize)
        .await?;
    assert_eq!(content, expected);
    Ok(())
}

pub async fn write_multiple_then_list<ST: Storage>(
    storage: &TestingStorage<ST>,
) -> anyhow::Result<()> {
    let repository = Uuid::new_v4();
    let paths = vec![
        StoragePath::from("/hello/world"),
        StoragePath::from("/hello/nitro_repo"),
        StoragePath::from("/hello/there"),
        StoragePath::from("/hello/this/item"),
        StoragePath::from("/hello/this/storage"),
    ];

    let content = FileContent::from("Hello, World!");

    for path in paths.iter() {
        let (_, _) = storage.save_file(repository, content.clone(), path).await?;
    }
    //let expected: Vec<u8> = content.try_into()?;

    let read_content = storage
        .open_file(repository, &StoragePath::from("/hello"))
        .await?;

    assert!(read_content.is_some(), "/hello not found");
    let read_content = read_content.unwrap();
    assert!(read_content.is_directory(), "File is not a file");

    let (files, meta) = read_content.directory().unwrap();
    debug!(?meta, "Directory Meta");
    debug!(?files, "Files");
    assert_eq!(files.len(), 4, "The number of files is incorrect");
    assert_eq!(
        meta.file_type.file_count,
        files.len() as u64,
        "The file count is incorrect"
    );

    for file in files {
        debug!(?file, "Item in directory");
    }
    Ok(())
}

pub async fn should_conflict<ST: Storage>(storage: &TestingStorage<ST>) -> anyhow::Result<()> {
    let repository = Uuid::new_v4();

    let content = FileContent::from("Hello, World!");

    storage
        .save_file(repository, content.clone(), &StoragePath::from("/a/b"))
        .await?;

    let Err(error) = storage
        .save_file(repository, content.clone(), &StoragePath::from("/a/b/c"))
        .await
    else {
        panic!("Expected error, but got success");
    };
    let storage_error: StorageError = error.into();
    info!(?storage_error, "Error");
    Ok(())
}
