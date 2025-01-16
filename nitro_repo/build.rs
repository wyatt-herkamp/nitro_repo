#![allow(dead_code)]
use std::{
    env,
    fs::File,
    io::{prelude::*, Seek, Write},
    iter::Iterator,
    path::{Path, PathBuf},
};

use anyhow::Context;
use walkdir::{DirEntry, WalkDir};
use zip::{write::SimpleFileOptions, ZipWriter};

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "frontend")]
    build_frontend()?;
    Ok(())
}
fn build_frontend() -> anyhow::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .with_context(|| "CARGO_MANIFEST_DIR not set")?
        .parent()
        .context("Invalid CARGO_MANIFEST_DIR. (Could not get parent)")?
        .to_path_buf();
    let frontend_src = manifest_dir.join("site");
    if !frontend_src.exists() {
        return Err(anyhow::anyhow!("site directory not found"));
    }

    println!("cargo::rerun-if-changed={}", manifest_dir.display());
    zip_site(frontend_src)?;
    Ok(())
}

/// Bundling files seem to be broken with Android. So as a work around. I will zip the files and include them in the binary.
fn zip_site(frontend: impl AsRef<Path>) -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").with_context(|| "OUT_DIR not set")?;
    let src = frontend.as_ref().join("dist");
    if !src.exists() {
        return Err(anyhow::anyhow!("site build directory not found"));
    }
    let dst = PathBuf::from(out_dir).join("frontend.zip");
    if dst.exists() {
        std::fs::remove_file(&dst)?;
    }
    let file = File::create(&dst)?;

    let walkdir = WalkDir::new(&src);
    let it = walkdir.into_iter();

    internal_zip_dir(
        &mut it.filter_map(|e| e.ok()),
        &src,
        file,
        zip::CompressionMethod::Stored,
    )?;
    println!("cargo:rustc-env=FRONTEND_ZIP={}", dst.display());
    println!("cargo:rustc-env=FRONTEND_SRC={}", src.display());

    Ok(())
}
fn internal_zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> anyhow::Result<()>
where
    T: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::with_capacity(1024);
    for entry in it {
        let absolute_path = entry.path();
        let stripped_path = entry.path().strip_prefix(prefix)?;
        let name = camino::Utf8Path::from_path(stripped_path)
            .with_context(|| format!("{stripped_path:?} Could not be converted to UTF-8"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if absolute_path.is_file() {
            zip.start_file(name.as_str(), options)?;
            let mut f = File::open(absolute_path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_str().is_empty() {
            zip.add_directory(name.to_string(), options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}
