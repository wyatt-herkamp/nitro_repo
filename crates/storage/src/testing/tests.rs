use nr_core::storage::StoragePath;
use tracing::{debug, info};
use uuid::Uuid;

use super::storage::TestingStorage;
use crate::{FileContent, FileType, Storage, StorageError, StorageFile, meta::RepositoryMeta};
pub async fn full_test<ST: Storage>(storage: TestingStorage<ST>) -> anyhow::Result<()>
where
    ST::DirectoryStream: 'static,
{
    write_then_read(&storage).await?;
    write_multiple_then_list(&storage).await?;
    should_conflict(&storage).await?;
    repository_meta_round_trip(&storage).await?;
    file_information_is_accurate(&storage).await?;
    overwrite_preserves_repository_meta(&storage).await?;
    delete_removes_file(&storage).await?;
    stream_directory_lists_children(&storage).await?;
    storage.unload().await?;
    Ok(())
}

/// Repository metadata must survive a write/read cycle.
///
/// Maven stamps project and version ids onto stored paths through this, and resolves them back on
/// every read. It was previously untested for every backend — the harness's own implementations
/// of these two methods were `todo!()` — which is how S3 shipped with both of them unimplemented.
pub async fn repository_meta_round_trip<ST: Storage>(
    storage: &TestingStorage<ST>,
) -> anyhow::Result<()> {
    let repository = Uuid::new_v4();
    let path = StoragePath::from("meta/target.txt");
    storage
        .save_file(repository, FileContent::from("content"), &path)
        .await?;

    assert_eq!(
        storage.get_repository_meta(repository, &path).await?,
        Some(RepositoryMeta::default()),
        "A freshly written file should have empty repository meta, not none"
    );

    let mut meta = RepositoryMeta::default();
    meta.set_project_id(Uuid::new_v4());
    meta.set_version_id(Uuid::new_v4());
    meta.insert("hello", "world");
    storage
        .put_repository_meta(repository, &path, meta.clone())
        .await?;

    assert_eq!(
        storage.get_repository_meta(repository, &path).await?,
        Some(meta),
        "Repository meta did not round trip"
    );
    Ok(())
}

/// `get_file_information` must report the real size and hashes, not placeholders.
///
/// The S3 backend used to fabricate this — every file reported `file_size: 0` with no hashes —
/// which silently broke Content-Length and ETag on every artifact served from a bucket.
pub async fn file_information_is_accurate<ST: Storage>(
    storage: &TestingStorage<ST>,
) -> anyhow::Result<()> {
    let repository = Uuid::new_v4();
    let path = StoragePath::from("info/artifact.jar");
    let body = "Some artifact content";
    storage
        .save_file(repository, FileContent::from(body), &path)
        .await?;

    let info = storage
        .get_file_information(repository, &path)
        .await?
        .expect("File information missing for a file that was just written");

    let FileType::File(file_type) = info.file_type else {
        panic!("Expected a file, got {:?}", info.file_type);
    };
    assert_eq!(
        file_type.file_size,
        body.len() as u64,
        "Reported size does not match what was written"
    );
    assert!(
        file_type.file_hash.sha2_256.is_some(),
        "No sha2-256 recorded; the ETag header depends on this"
    );

    let directory = storage
        .get_file_information(repository, &StoragePath::from("info/"))
        .await?
        .expect("File information missing for the parent directory");
    assert!(
        matches!(directory.file_type, FileType::Directory(_)),
        "Parent of a file should report as a directory, got {:?}",
        directory.file_type
    );
    Ok(())
}

/// Overwriting an artifact must not silently discard the metadata attached to its path.
pub async fn overwrite_preserves_repository_meta<ST: Storage>(
    storage: &TestingStorage<ST>,
) -> anyhow::Result<()> {
    let repository = Uuid::new_v4();
    let path = StoragePath::from("overwrite/artifact.pom");
    storage
        .save_file(repository, FileContent::from("first"), &path)
        .await?;

    let mut meta = RepositoryMeta::default();
    let project_id = Uuid::new_v4();
    meta.set_project_id(project_id);
    storage.put_repository_meta(repository, &path, meta).await?;

    storage
        .save_file(repository, FileContent::from("second, longer"), &path)
        .await?;

    let after = storage
        .get_repository_meta(repository, &path)
        .await?
        .expect("Repository meta vanished after an overwrite");
    assert_eq!(
        after.project_id,
        Some(project_id),
        "Overwriting an artifact dropped the project id attached to its path"
    );
    Ok(())
}

/// Deleting must remove the file and stop reporting it as present.
pub async fn delete_removes_file<ST: Storage>(storage: &TestingStorage<ST>) -> anyhow::Result<()> {
    let repository = Uuid::new_v4();
    let path = StoragePath::from("delete/me.txt");
    storage
        .save_file(repository, FileContent::from("temporary"), &path)
        .await?;
    assert!(storage.file_exists(repository, &path).await?);

    assert!(
        storage.delete_file(repository, &path).await?,
        "Deleting an existing file should report that it did something"
    );
    assert!(
        !storage.file_exists(repository, &path).await?,
        "File still exists after being deleted"
    );
    assert!(
        storage.open_file(repository, &path).await?.is_none(),
        "Deleted file can still be opened"
    );
    Ok(())
}

/// `stream_directory` must yield a directory's children, and nothing for a path that is not one.
///
/// This is what the browse websocket reads; it was `todo!()` on S3.
pub async fn stream_directory_lists_children<ST: Storage>(
    storage: &TestingStorage<ST>,
) -> anyhow::Result<()>
where
    ST::DirectoryStream: 'static,
{
    let repository = Uuid::new_v4();
    let names = ["one.txt", "two.txt", "three.txt"];
    for name in names {
        storage
            .save_file(
                repository,
                FileContent::from("content"),
                &StoragePath::from(format!("streamed/{name}")),
            )
            .await?;
    }

    let stream = storage
        .stream_directory(repository, &StoragePath::from("streamed/"))
        .await?
        .expect("No stream for a directory that exists");
    let files = crate::streaming::collect_directory_stream(stream).await?;
    assert_eq!(
        files.len(),
        names.len(),
        "Streamed directory returned the wrong number of entries: {files:?}"
    );

    let mut listed: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();
    listed.sort_unstable();
    let mut expected = names.to_vec();
    expected.sort_unstable();
    assert_eq!(
        listed, expected,
        "Streamed directory listed the wrong names"
    );

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
    let paths = [
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
