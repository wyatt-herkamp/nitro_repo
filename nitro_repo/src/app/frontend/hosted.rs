use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

use axum::{
    extract::{Request, State},
    response::Response,
};
use handlebars::Handlebars;

use tracing::{debug, instrument, trace, warn};

use crate::{app::NitroRepo, error::InternalError, utils::response_builder::ResponseBuilder};

use super::FrontendError;

#[cfg(feature = "frontend")]
static FRONTEND_DATA: &[u8] = include_bytes!(env!("FRONTEND_ZIP"));
#[derive(Debug)]
pub struct HostedFrontend {
    pub frontend_path: PathBuf,
    pub enabled: AtomicBool,
    pub handlebars: Handlebars<'static>,
}
impl HostedFrontend {
    pub fn new(frontend_path: Option<PathBuf>) -> Result<Self, FrontendError> {
        let handlebars = Handlebars::new();
        let frontend_path = frontend_path.unwrap_or_else(|| Path::new("frontend").to_owned());
        Self::save_frontend(frontend_path.clone())?;
        let frontend = Self {
            frontend_path,
            enabled: AtomicBool::new(true),
            handlebars: handlebars,
        };
        Ok(frontend)
    }
    fn save_frontend(frontend_path: PathBuf) -> Result<(), FrontendError> {
        use std::{
            fs::{self, remove_dir_all, remove_file, File},
            io,
        };

        use zip::ZipArchive;

        // Ensure Directory Is Created
        if !frontend_path.exists() {
            std::fs::create_dir(&frontend_path)?;
        }
        // Ensure Directory Is Empty
        for entry in std::fs::read_dir(&frontend_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                remove_dir_all(&path)?;
            } else {
                remove_file(&path)?;
            }
        }
        let reader = std::io::Cursor::new(FRONTEND_DATA);
        let mut archive = ZipArchive::new(reader)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => frontend_path.join(path),
                None => continue,
            };

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    debug!("File {i} comment: {comment}");
                }
            }

            if (*file.name()).ends_with('/') {
                debug!("File {} extracted to \"{}\"", i, outpath.display());
                std::fs::create_dir_all(&outpath)?;
            } else {
                debug!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }
        Ok(())
    }
    pub fn does_path_exist(&self, path: &str) -> bool {
        let path = self.get_path(path);
        trace!(?path, "Checking Path");
        path.exists()
    }
    pub fn get_path(&self, path: &str) -> PathBuf {
        let path = if path.starts_with("/") {
            &path[1..]
        } else {
            path
        };
        self.frontend_path.join(path)
    }

    pub fn get_index_file(&self, site: &NitroRepo) -> Result<Vec<u8>, FrontendError> {
        let path = self.frontend_path.join("index.html");
        if !path.exists() {
            return Err(FrontendError::IndexPageMissing);
        }
        let content = std::fs::read_to_string(path)?;
        let instance = site.instance.lock().clone();
        let rendered = self.handlebars.render_template(&content, &instance)?;
        Ok(rendered.into_bytes())
    }

    pub fn get_file_as_response(&self, path: &str) -> Result<Response, FrontendError> {
        let path = self.get_path(path);
        if !path.exists() {
            warn!(?path, "File Not Found");
            return Ok(ResponseBuilder::not_found().empty());
        }
        let content_type = Self::guess_mime_type(&path);
        let file = std::fs::read(path)?;
        Ok(ResponseBuilder::ok().content_type(&content_type).body(file))
    }
    #[instrument]
    fn guess_mime_type(path: &Path) -> String {
        match mime_guess::from_path(path).first() {
            Some(mime) => {
                let mime: &str = mime.as_ref();
                debug!(?mime, "Mime Type");
                mime.to_owned()
            }
            None => "text/plain".to_owned(),
        }
    }
}

#[instrument]
pub async fn frontend_request(
    State(site): State<NitroRepo>,
    request: Request,
) -> Result<Response, InternalError> {
    let frontend = &site.frontend;
    let path = request.uri().path();
    debug!(?path, "Frontend Request");

    if path.eq("/") || path.eq("") || path.eq("/index.html") || (!frontend.does_path_exist(path)) {
        if should_return_404(path) {
            return Ok(ResponseBuilder::not_found().empty());
        }
        debug!("Returning Index File");
        let index_file = frontend.get_index_file(&site)?;
        return Ok(ResponseBuilder::ok().html(index_file));
    }
    let response = frontend.get_file_as_response(path)?;
    Ok(response)
}
/// Basically if it contains and extension that we want to send a server side 404 from.
///
/// Such as images, css, js, etc.
fn should_return_404(path: &str) -> bool {
    let extensions = vec![
        ".css", ".js", ".png", ".jpg", ".jpeg", ".svg", ".ico", ".webp", ".gif", ".ttf", ".ico",
    ];
    for ext in extensions {
        if path.ends_with(ext) {
            return true;
        }
    }
    false
}
