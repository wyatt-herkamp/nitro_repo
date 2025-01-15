use bytes::{Bytes, BytesMut};
use derive_more::derive::From;
use http_body::{Body, Frame};
use std::fs::File as SyncFile;
use std::{
    fmt::Debug,
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncReadExt;
use tokio::{fs::File, io::AsyncRead};
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
    Bytes(FileContentBytes),
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
            StorageFileReader::Bytes(bytes) => return Ok(bytes.into()),
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
            StorageFileReader::Bytes(bytes) => {
                let len = std::cmp::min(buf.remaining(), bytes.len());
                buf.put_slice(&bytes.as_ref()[..len]);
                Poll::Ready(Ok(()))
            }
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
