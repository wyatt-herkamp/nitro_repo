use std::fmt::Write;

use http::request::Parts;
use nr_core::storage::StoragePath;
use nr_storage::Storage;

use super::CargoRegistryError;
use crate::repository::Repository;

/// Lowercase hex, which is the only form cargo accepts for `cksum`.
///
/// `sha2` 0.11 returns a `digest::Output`, which does not implement `LowerHex`, so this is written
/// out rather than formatted. The same helper exists in the Maven module for the same reason.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
}

/// The directory holding every version of a crate, relative to the repository root.
///
/// This is also the project's `path`, which is what lets browsing it resolve to the project page —
/// see [`CargoHostedRegistry::resolve_project_and_version_for_path`]. `crates/` keeps the files
/// clear of the `index/` tree a future on-disk index would want.
///
/// [`CargoHostedRegistry::resolve_project_and_version_for_path`]: super::hosted::CargoHostedRegistry
pub fn crate_project_dir(name: &str) -> Result<StoragePath, CargoRegistryError> {
    let mut path = StoragePath::parse("crates")?;
    path.push_mut(name);
    Ok(path)
}

/// The directory holding one version of a crate.
///
/// A version gets a directory of its own rather than sitting as a lone file in the crate's, so that
/// its path is something `project_versions.path` can name and browsing it resolves to the version.
/// It is also the layout Maven and npm already use.
pub fn crate_version_dir(name: &str, version: &str) -> Result<StoragePath, CargoRegistryError> {
    let mut path = crate_project_dir(name)?;
    path.push_mut(version);
    Ok(path)
}

/// Where a crate's `.crate` file is stored.
pub fn crate_file_path(name: &str, version: &str) -> Result<StoragePath, CargoRegistryError> {
    let mut path = crate_version_dir(name, version)?;
    path.push_mut(&format!("{name}-{version}.crate"));
    Ok(path)
}

/// The absolute URL cargo should use as the root of this registry.
///
/// A registry can be reached two ways, and `config.json` has to name the one the client actually
/// used. A request that arrived on a hostname registered to this repository is answered with that
/// hostname's root — telling such a client to go back through `/repositories/{storage}/{name}`
/// would work but would undo the point of the custom domain, and would break outright when the
/// custom host is the only one reachable from where cargo is running.
pub fn registry_base_url(
    repository: &impl Repository,
    parts: &Parts,
) -> Result<String, CargoRegistryError> {
    let site = repository.site();

    if let Some(host) = crate::app::host_routing::request_host(
        &parts.headers,
        &parts.uri,
        site.general_security_settings.trust_forwarded_host,
    ) && site
        .repository_for_hostname(&host)
        .is_some_and(|found| found.id() == repository.id())
        && let Some(origin) =
            crate::app::host_routing::request_origin(&site, &parts.headers, &parts.uri)
    {
        // The origin, not the looked-up host: the lookup is deliberately port-blind, but the URL
        // written into `config.json` is what cargo will actually request.
        return Ok(origin);
    }

    let base = {
        let instance = site.instance.lock();
        instance.app_url.trim_end_matches('/').to_owned()
    };
    if base.is_empty() {
        return Err(CargoRegistryError::NoAppUrl);
    }
    let storage = repository.get_storage();
    let storage_name = storage
        .storage_config()
        .storage_config
        .storage_name
        .to_string();
    Ok(format!(
        "{base}/repositories/{storage_name}/{}",
        repository.name()
    ))
}
