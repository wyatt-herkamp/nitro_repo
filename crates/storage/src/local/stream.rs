use std::{path::PathBuf, task::Poll};

use futures::Stream;
use pin_project::pin_project;
use tokio::fs::ReadDir;
use tokio_stream::wrappers::ReadDirStream;
use tracing::trace;

use crate::{
    is_hidden_file, local::error::LocalStorageError, streaming::DirectoryListStream,
    DirectoryFileType, FileFileType, FileType, StorageError, StorageFileMeta,
};

#[derive(Debug)]
#[pin_project]
pub struct LocalDirectoryListStream {
    #[pin]
    files: FileOrDirectory,
    meta: StorageFileMeta<FileType>,
}
impl LocalDirectoryListStream {
    pub fn new_directory(read_dir: ReadDir, meta: StorageFileMeta<DirectoryFileType>) -> Self {
        LocalDirectoryListStream {
            files: FileOrDirectory::Directory(ReadDirStream::new(read_dir)),
            meta: meta.map_type(FileType::Directory),
        }
    }
    pub fn new_file(file_path: PathBuf, meta: StorageFileMeta<FileFileType>) -> Self {
        LocalDirectoryListStream {
            files: FileOrDirectory::File(Some(file_path)),
            meta: meta.map_type(FileType::File),
        }
    }
}
impl DirectoryListStream for LocalDirectoryListStream {
    fn number_of_files(&self) -> u64 {
        match &self.meta.file_type() {
            FileType::Directory(dir) => dir.file_count,
            _ => 1,
        }
    }
}
#[derive(Debug)]
#[pin_project(project = FileOrDirectoryProj)]
enum FileOrDirectory {
    Directory(#[pin] ReadDirStream),
    File(Option<PathBuf>),
}

impl Stream for LocalDirectoryListStream {
    type Item = Result<Option<StorageFileMeta<FileType>>, StorageError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();

        let next_file = {
            let this_files = this.files.project();
            match this_files {
                FileOrDirectoryProj::Directory(read_dir_stream) => {
                    match read_dir_stream.poll_next(cx) {
                        Poll::Ready(ready) => ready.map(|entry| entry.map(|entry| entry.path())),
                        Poll::Pending => return Poll::Pending,
                    }
                }
                FileOrDirectoryProj::File(path_buf) => {
                    let path = path_buf.take();

                    path.map(Ok)
                }
            }
        };
        let Some(entry) = next_file else {
            return std::task::Poll::Ready(None);
        };

        let path = entry.map_err(LocalStorageError::from)?;

        if path.is_file() && is_hidden_file(&path) {
            // I am not really sure what to do with hidden files in a stream?
            // I am guessing if return Poll::Pending now it will just skip the file?
            trace!(?path, "Skipping Meta File");
            return Poll::Ready(Some(Ok(None)));
        }
        let file_meta = StorageFileMeta::read_from_path(&path)
            .map(Some)
            .map_err(StorageError::from);
        Poll::Ready(Some(file_meta))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let number_of_files = self.number_of_files();
        (number_of_files as usize, Some(number_of_files as usize))
    }
}
