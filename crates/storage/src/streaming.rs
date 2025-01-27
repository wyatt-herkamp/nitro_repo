use std::{fmt::Debug, pin::Pin};

use futures::Stream;
use pin_project::pin_project;

use crate::{local::LocalStorage, DirectoryFileType, FileType, StorageError, StorageFileMeta};
/// Why is it Result<Option<StorageFileMeta<FileType>>>?
///
/// If a file is skipped it will return Ok(None)

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
