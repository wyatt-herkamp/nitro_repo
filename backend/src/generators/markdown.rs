use crate::error::internal_error::InternalError;
use crate::generators::GeneratorCache;
use comrak::{parse_document, Arena};
use log::error;
use std::path::PathBuf;
use std::sync::Arc;

pub fn parse_to_html(
    markdown: impl AsRef<str>,
    path: PathBuf,
    cache: Arc<GeneratorCache>,
) -> Result<Vec<u8>, InternalError> {
    let options = comrak::ExtensionOptionsBuilder::default()
        .strikethrough(true)
        .table(true)
        .tagfilter(true)
        .tasklist(true)
        .autolink(true)
        .build()
        .unwrap();
    let render_options = comrak::RenderOptions::default();
    let parse_options = comrak::ParseOptions::default();
    let options = comrak::Options {
        extension: options,
        render: render_options,
        parse: parse_options,
    };
    let arena = Arena::new();
    let html = parse_document(&arena, markdown.as_ref(), &options);
    let mut content = Vec::with_capacity(markdown.as_ref().len());
    comrak::format_html(html, &options, &mut content)?;
    let clone = content.clone();
    tokio::spawn(async move {
        if let Err(error) = cache.push_to_cache(path, clone).await {
            error!("Failed to push to cache. {:?}", error);
        }
    });
    Ok(content)
}
