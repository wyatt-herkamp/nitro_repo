use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

use axum::{
    extract::{Request, State},
    response::Response,
};
use handlebars::Handlebars;

use http::StatusCode;
use serde::Deserialize;
use tracing::{debug, instrument, trace, warn};

use crate::{app::NitroRepo, error::InternalError, utils::response_builder::ResponseBuilder};

use super::FrontendError;

#[cfg(feature = "frontend")]
static FRONTEND_DATA: &[u8] = include_bytes!(env!("FRONTEND_ZIP"));
#[derive(Debug, Clone, Deserialize)]
pub struct RouteItem {
    pub path: FrontendRoute,
    pub name: String,
}
#[derive(Debug)]
pub struct HostedFrontend {
    pub frontend_path: PathBuf,
    pub enabled: AtomicBool,
    pub routes: Vec<RouteItem>,
    pub handlebars: Handlebars<'static>,
}
impl HostedFrontend {
    pub fn new(frontend_path: Option<PathBuf>) -> Result<Self, FrontendError> {
        let handlebars = Handlebars::new();
        let frontend_path = frontend_path.unwrap_or_else(|| Path::new("frontend").to_owned());
        Self::save_frontend(frontend_path.clone())?;
        let mut frontend = Self {
            frontend_path,
            enabled: AtomicBool::new(true),
            routes: vec![],
            handlebars: handlebars,
        };
        frontend.read_routes()?;
        Ok(frontend)
    }
    fn read_routes(&mut self) -> Result<(), FrontendError> {
        let path = self.frontend_path.join("routes.json");
        if !path.exists() {
            return Err(FrontendError::FileNotFound);
        }
        let routes: Vec<RouteItem> = serde_json::from_slice(&std::fs::read(path)?)?;
        debug!(?routes, "Routes registered");
        self.routes = routes;
        Ok(())
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
        debug!(?instance, "Instance");
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
        let status_code = if path.eq("/")
            || path.eq("")
            || frontend
                .routes
                .iter()
                .any(|route| route.path.matches_path(path))
        {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        };
        debug!("Returning Index File");
        let index_file = frontend.get_index_file(&site)?;
        return Ok(ResponseBuilder::default()
            .status(status_code)
            .html(index_file));
    }
    let response = frontend.get_file_as_response(path)?;
    Ok(response)
}
/// Basically if it contains and extension that we want to send a server side 404 from with no content
///
/// Such as images, css, js, etc.
fn should_return_404(path: &str) -> bool {
    if path.starts_with("/assets") {
        return true;
    }
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
#[derive(Debug, Clone)]

pub struct FrontendRoute {
    parts: Vec<FrontendRouteComponent>,
    has_catch_all: bool,
}
impl<'de> Deserialize<'de> for FrontendRoute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FrontendRoute::try_from(s).map_err(serde::de::Error::custom)
    }
}
impl FrontendRoute {
    pub fn matches_path(&self, path: &str) -> bool {
        let path = path.trim_start_matches('/');
        let path = path.trim_end_matches('/');
        let split_path: Vec<_> = path.split('/').collect();
        trace!("{:?}", split_path);

        if split_path.len() != self.parts.len() && !self.has_catch_all {
            debug!("Path Lengths Do Not Match");
            return false;
        }

        for (part, component) in split_path.iter().zip(self.parts.iter()) {
            match component {
                FrontendRouteComponent::String(string) => {
                    if string != part {
                        debug!(expected = ?string, actual = ?part, "Path Component Mismatch");
                        return false;
                    }
                }
                FrontendRouteComponent::Param {
                    key: _,
                    optional,
                    catch_all,
                } => {
                    if *catch_all {
                        return true;
                    }
                    if !optional && part.is_empty() {
                        debug!(expected = ?part, "Path Component Mismatch");
                        return false;
                    }
                }
            }
        }
        true
    }
}
impl TryFrom<String> for FrontendRoute {
    type Error = FrontendError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let split_parts = value.split('/');
        let mut has_catch_all = false;
        let mut parts = Vec::with_capacity(split_parts.size_hint().0);
        for part in split_parts {
            if part.starts_with(':') {
                let optional = part.ends_with('?');
                let catch_all = part.contains("(.*)");
                let key = part[1..].trim_end_matches('?').replace("(.*)", "");
                let key = if key.contains("(") {
                    let mut new_key = String::with_capacity(part.len() - 2);
                    let mut found_paren = false;
                    for ele in key.chars() {
                        if ele == '(' {
                            found_paren = true;
                            continue;
                        } else if found_paren && ele == ')' {
                            found_paren = false;
                            continue;
                        }
                        if !found_paren {
                            new_key.push(ele);
                        }
                    }
                    new_key
                } else {
                    key.to_owned()
                };
                if catch_all {
                    has_catch_all = true;
                }
                parts.push(FrontendRouteComponent::Param {
                    key,
                    optional,
                    catch_all,
                });
            } else {
                if part.is_empty() {
                    continue;
                }
                parts.push(FrontendRouteComponent::String(part.to_owned()));
            }
        }
        Ok(Self {
            parts,
            has_catch_all,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendRouteComponent {
    String(String),
    Param {
        key: String,
        optional: bool,
        catch_all: bool,
    },
}
#[cfg(test)]
mod route_parser_tests {

    use super::*;
    #[test]
    pub fn basic_test() {
        let route = FrontendRoute::try_from("/test/:id".to_string()).unwrap();
        println!("{:?}", route);
        assert_eq!(route.parts.len(), 2);
        assert_eq!(
            route.parts[0],
            FrontendRouteComponent::String("test".to_string())
        );
        assert_eq!(
            route.parts[1],
            FrontendRouteComponent::Param {
                key: "id".to_string(),
                optional: false,
                catch_all: false
            }
        );

        assert!(route.matches_path("/test/123"));
        assert!(route.matches_path("/test/123/"));

        assert!(!route.matches_path("/test/"));
    }
    #[test]
    pub fn browse_test() {
        let route = FrontendRoute::try_from("/browse/:id/:catchAll(.*)?".to_string()).unwrap();
        println!("{:?}", route);
        assert_eq!(route.parts.len(), 3);
        assert_eq!(
            route.parts[0],
            FrontendRouteComponent::String("browse".to_string())
        );
        assert_eq!(
            route.parts[1],
            FrontendRouteComponent::Param {
                key: "id".to_string(),
                optional: false,
                catch_all: false
            }
        );
        assert_eq!(
            route.parts[2],
            FrontendRouteComponent::Param {
                key: "catchAll".to_string(),
                optional: true,
                catch_all: true
            }
        );
        assert!(route.matches_path("/browse/123/456"));
        assert!(route.matches_path("/browse/123/456/"));
        assert!(route.matches_path("/browse/123/456/789"));
        assert!(route.matches_path("/browse/123/456/789/"));
        assert!(!route.matches_path("/not_browse/"));
    }

    #[test]
    fn parse_all() {
        let file = include_str!("../../../../site/src/router/routes.json");
        let routes: Vec<RouteItem> = serde_json::from_str(file).unwrap();

        for route in routes {
            println!("{:?}", route);
        }
    }
}
