use bytes::Bytes;
use futures::Stream;

pub struct BrowseStream<S: Stream<Item = reqwest::Result<Bytes>>> {
    pub stream: S,
}
impl<S> Stream for BrowseStream<S>
where
    S: Stream<Item = reqwest::Result<Bytes>>,
{
    type Item = reqwest::Result<Bytes>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}
