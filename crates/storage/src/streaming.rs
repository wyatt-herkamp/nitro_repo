use std::{fmt::Debug, pin::Pin};

use futures::{Stream, StreamExt};
use pin_project::pin_project;

use crate::{DirectoryFileType, FileType, StorageError, StorageFileMeta};
/// Why is it Result<Option<StorageFileMeta<FileType>>>?
///
/// If a file is a hidden file it will return Ok(None)
pub trait DirectoryListStream:
    Stream<Item = Result<Option<StorageFileMeta<FileType>>, StorageError>> + Debug + Send
{
    fn number_of_files(&self) -> u64;
}
#[derive(Debug)]
#[pin_project]
pub struct VecDirectoryListStream {
    files: std::vec::IntoIter<StorageFileMeta<FileType>>,
    directory_meta: StorageFileMeta<DirectoryFileType>,
}
impl VecDirectoryListStream {
    pub fn new(
        files: Vec<StorageFileMeta<FileType>>,
        directory_meta: StorageFileMeta<DirectoryFileType>,
    ) -> Self {
        VecDirectoryListStream {
            files: files.into_iter(),
            directory_meta,
        }
    }
}
impl Stream for VecDirectoryListStream {
    type Item = Result<Option<StorageFileMeta<FileType>>, StorageError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        match this.files.next() {
            Some(file) => std::task::Poll::Ready(Some(Ok(Some(file)))),
            None => std::task::Poll::Ready(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.files.len(), Some(self.files.len()))
    }
}
#[derive(Debug)]
pub struct EmptyDirectoryListStream;

impl Stream for EmptyDirectoryListStream {
    type Item = Result<Option<StorageFileMeta<FileType>>, StorageError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(None)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl DirectoryListStream for EmptyDirectoryListStream {
    fn number_of_files(&self) -> u64 {
        0
    }
}
impl DirectoryListStream for VecDirectoryListStream {
    fn number_of_files(&self) -> u64 {
        self.directory_meta.file_type().file_count
    }
}
#[derive(Debug)]
#[pin_project]
pub struct DynDirectoryListStream {
    #[pin]
    stream: Pin<Box<dyn DirectoryListStream + Send + 'static>>,
}
impl DynDirectoryListStream {
    pub fn new<S>(stream: S) -> Self
    where
        S: DirectoryListStream + Send + 'static,
    {
        DynDirectoryListStream {
            stream: Box::pin(stream),
        }
    }
}
impl Stream for DynDirectoryListStream {
    type Item = Result<Option<StorageFileMeta<FileType>>, StorageError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.poll_next(cx)
    }
}

impl DirectoryListStream for DynDirectoryListStream {
    fn number_of_files(&self) -> u64 {
        self.stream.number_of_files()
    }
}
/// Collects all files from a directory stream
pub async fn collect_directory_stream<Stream>(
    stream: Stream,
) -> Result<Vec<StorageFileMeta<FileType>>, StorageError>
where
    Stream: DirectoryListStream + Send + 'static,
{
    let mut files = Vec::with_capacity(stream.number_of_files() as usize);
    let mut stream = Box::pin(stream);
    while let Some(file) = stream.next().await {
        if let Some(file) = file? {
            files.push(file);
        }
    }
    Ok(files)
}
