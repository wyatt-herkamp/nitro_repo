use std::{
    pin::Pin,
    task::{Context, Poll, ready},
    time::Instant,
};

use http_body::{Body, Frame};
use opentelemetry::KeyValue;
use pin_project::pin_project;
use tracing::Span;

use crate::app::NitroRepo;

use super::request_span;

#[pin_project]
pub struct TraceResponseBody {
    #[pin]
    pub(crate) inner: axum::body::Body,
    pub(crate) start: Instant,
    pub(crate) span: Span,
    pub(crate) state: NitroRepo,
    pub(crate) attributes: Vec<KeyValue>,
    pub(crate) total_bytes: u64,
}

impl Body for TraceResponseBody {
    type Data = axum::body::Bytes;
    type Error = axum::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = ready!(this.inner.poll_frame(cx));

        *this.start = Instant::now();

        match result {
            Some(Ok(frame)) => {
                let frame = match frame.into_data() {
                    Ok(chunk) => {
                        *this.total_bytes += chunk.len() as u64;
                        Frame::data(chunk)
                    }
                    Err(frame) => frame,
                };

                let frame = match frame.into_trailers() {
                    Ok(trailers) => Frame::trailers(trailers),
                    Err(frame) => frame,
                };

                Poll::Ready(Some(Ok(frame)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => {
                this.state
                    .metrics
                    .response_size_bytes
                    .record(*this.total_bytes, this.attributes);
                request_span::on_end_of_stream(*this.total_bytes, this.span);
                Poll::Ready(None)
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
