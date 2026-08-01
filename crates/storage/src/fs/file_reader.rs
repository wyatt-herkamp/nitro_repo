use std::{
    fmt::Debug,
    fs::File as SyncFile,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use derive_more::derive::From;
use http_body::{Body, Frame};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt},
};
use tokio_util::io::poll_read_buf;

use super::FileContentBytes;

/// StorageFileReader is a wrapper around different types of readers.
#[derive(From)]
pub enum StorageFileReader {
    /// File Readers will be the most common type of reader.
    /// For this reason, we will give it a special variant. To prevent dynamic dispatch.
    File(File),
    /// An Async Reader type. This will be used for remote storage. Such as S3.
    AsyncReader(Pin<Box<dyn tokio::io::AsyncRead + Send>>),
    /// Content already in memory.
    ///
    /// Held in a cursor because [AsyncRead::poll_read] has to remember how much it has already
    /// handed out. Without that the reader copies the same prefix on every poll and never reports
    /// end-of-stream, so a response body built from it never terminates.
    Bytes(io::Cursor<FileContentBytes>),
}
impl From<FileContentBytes> for StorageFileReader {
    fn from(bytes: FileContentBytes) -> Self {
        StorageFileReader::Bytes(io::Cursor::new(bytes))
    }
}
impl StorageFileReader {
    pub async fn read_to_vec(self, size_hint: usize) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(size_hint);
        match self {
            StorageFileReader::File(mut file) => {
                file.read_to_end(&mut buf).await?;
            }
            StorageFileReader::AsyncReader(mut reader) => {
                tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut buf).await?;
            }
            StorageFileReader::Bytes(cursor) => {
                // Reads from wherever the cursor is, so this stays consistent with `poll_read`
                // if anything has already been taken from the reader.
                let position = cursor.position() as usize;
                let inner = cursor.into_inner();
                if position == 0 {
                    return Ok(inner.into());
                }
                let remaining = inner.as_ref();
                return Ok(remaining[position.min(remaining.len())..].to_vec());
            }
        }
        Ok(buf)
    }
}

impl From<SyncFile> for StorageFileReader {
    fn from(file: SyncFile) -> Self {
        StorageFileReader::File(File::from_std(file))
    }
}
impl StorageFileReader {
    pub fn into_body(self, capacity: usize) -> StorageFileReaderBody {
        StorageFileReaderBody {
            reader: Some(self),
            buf: BytesMut::with_capacity(capacity),
            capacity,
        }
    }
}
impl Debug for StorageFileReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageFileReader::File(_) => f.write_str("StorageFileReader::File"),
            StorageFileReader::AsyncReader(_) => f.write_str("StorageFileReader::AsyncReader"),
            StorageFileReader::Bytes(_) => f.write_str("StorageFileReader::Bytes"),
        }
    }
}
impl AsyncRead for StorageFileReader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            StorageFileReader::File(file) => Pin::new(file).poll_read(cx, buf),
            StorageFileReader::AsyncReader(reader) => Pin::new(reader).poll_read(cx, buf),
            StorageFileReader::Bytes(cursor) => Pin::new(cursor).poll_read(cx, buf),
        }
    }
}
#[pin_project::pin_project]
#[derive(Debug)]
pub struct StorageFileReaderBody {
    #[pin]
    reader: Option<StorageFileReader>,
    buf: BytesMut,
    capacity: usize,
}
impl Body for StorageFileReaderBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let mut this = self.as_mut().project();

        let reader = match this.reader.as_pin_mut() {
            Some(r) => r,
            None => return Poll::Ready(None),
        };

        if this.buf.capacity() == 0 {
            this.buf.reserve(*this.capacity);
        }

        match poll_read_buf(reader, cx, &mut this.buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => {
                self.project().reader.set(None);
                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(Ok(0)) => {
                self.project().reader.set(None);
                Poll::Ready(None)
            }
            Poll::Ready(Ok(_)) => {
                let chunk = this.buf.split();
                let frozen = chunk.freeze();
                Poll::Ready(Some(Ok(Frame::data(frozen))))
            }
        }
    }
    fn is_end_stream(&self) -> bool {
        self.reader.is_none()
    }
    fn size_hint(&self) -> http_body::SizeHint {
        let mut hint = http_body::SizeHint::default();
        // Capacity should be the size of the response.
        hint.set_lower(self.capacity as u64);
        hint
    }
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;

    use super::*;

    /// An in-memory reader must yield its content once and then stop.
    ///
    /// `poll_read` used to copy from the front of the buffer on every call without advancing, so
    /// a reader never reached end-of-stream and the body repeated the same bytes forever. A small
    /// body capacity here forces several polls, which is what exposes it.
    #[tokio::test]
    async fn bytes_reader_terminates() {
        let content = b"Hello, World! This content is longer than one chunk.".to_vec();
        let reader = StorageFileReader::from(FileContentBytes::Content(content.clone()));

        let collected = reader.into_body(8).collect().await.unwrap().to_bytes();

        assert_eq!(collected.as_ref(), content.as_slice());
    }

    /// Reading straight to a `Vec` must agree with streaming it.
    #[tokio::test]
    async fn bytes_reader_read_to_vec() {
        let content = b"Hello, World!".to_vec();
        let reader = StorageFileReader::from(FileContentBytes::Content(content.clone()));

        assert_eq!(reader.read_to_vec(content.len()).await.unwrap(), content);
    }

    /// An empty body should produce no frames rather than hanging.
    #[tokio::test]
    async fn empty_bytes_reader_terminates() {
        let reader = StorageFileReader::from(FileContentBytes::Content(Vec::new()));

        let collected = reader.into_body(8).collect().await.unwrap().to_bytes();

        assert!(collected.is_empty());
    }
}
