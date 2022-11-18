use crate::error::internal_error::InternalError;
use crate::generators::GeneratorCache;
use comrak::{parse_document, Arena, ComrakExtensionOptions, ComrakOptions, ComrakRenderOptions};
use log::error;
use std::path::PathBuf;
use std::sync::Arc;

pub fn parse_to_html(
    markdown: impl AsRef<str>,
    path: PathBuf,
    cache: Arc<GeneratorCache>,
) -> Result<Vec<u8>, InternalError> {
    let options = ComrakOptions {
        render: ComrakRenderOptions::default(),
        extension: ComrakExtensionOptions {
            autolink: true,
            strikethrough: true,
            table: true,
            tagfilter: true,
            tasklist: true,
            ..ComrakExtensionOptions::default()
        },
        ..ComrakOptions::default()
    };
    let arena = Arena::new();
    let html = parse_document(&arena, markdown.as_ref(), &options);
    let mut content = vec![];
    comrak::format_html(html, &options, &mut content)?;
    let clone = content.clone();
    tokio::spawn(async move {
        if let Err(error) = cache.push_to_cache(path, clone).await {
            error!("Failed to push to cache. {:?}", error);
        }
    });
    Ok(content)
}
