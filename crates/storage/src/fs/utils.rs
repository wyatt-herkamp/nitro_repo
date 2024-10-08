use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use chrono::{offset::LocalResult, DateTime, FixedOffset, Local, TimeZone};
use tracing::{error, instrument, warn};

use super::SerdeMime;

/// Converts a SystemTime to a DateTime<FixedOffset>.
///
/// The offset is based on the local timezone.
///
/// This function will return an error if the SystemTime is before the Unix Epoch.
/// This should not be possible, but it is handled just in case.
///
/// If the conversion is ambiguous, the earliest time is used.
pub fn system_time_to_date_time(time: std::time::SystemTime) -> io::Result<DateTime<FixedOffset>> {
    let time = time
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|v| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("SystemTime is before the Unix Epoch: {}", v),
            )
        })?
        .as_millis();
    // If this program is running when the unix epoch overflows the i64. I will be very impressed.

    match Local.timestamp_millis_opt(time as i64) {
        LocalResult::Single(ok) => Ok(ok.fixed_offset()),
        LocalResult::Ambiguous(earliest, latest) => {
            warn!(earliest= ?earliest, latest = ?latest,"Ambiguous time conversion. Using the earliest time");
            Ok(earliest.fixed_offset())
        }
        LocalResult::None => {
            error!("Could not convert SystemTime to DateTime. Duration {time}");
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Could not convert SystemTime to DateTime",
            ))
        }
    }
}

pub trait PathUtils {
    fn parent_or_err(&self) -> io::Result<&Path>;
    /// Appends an extension to the path.
    fn add_extension(&self, extension: &str) -> io::Result<PathBuf>;
}
impl PathUtils for PathBuf {
    fn parent_or_err(&self) -> io::Result<&Path> {
        self.parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Parent Directory not found"))
    }
    #[instrument]
    fn add_extension(&self, extension: &str) -> io::Result<PathBuf> {
        let mut path = self.clone();
        if path.extension().is_none() {
            path.set_extension(extension);
            return Ok(path);
        }
        let Some(old_extension) = path
            .extension()
            .and_then(|v| v.to_str().map(|v| v.to_owned()))
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path already has an extension",
            ));
        };
        path.set_extension(format!("{}.{}", old_extension, extension));
        Ok(path)
    }
}
#[instrument]
pub fn mime_type_for_file(file: &File, path: PathBuf) -> Option<SerdeMime> {
    if path.extension().unwrap_or_default() == "nr-meta" {
        return Some(SerdeMime(super::FILE_META_MIME));
    }
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    Some(SerdeMime(mime))
}

pub trait MetadataUtils {
    /// Get the creation time of the file as a DateTime<FixedOffset>.
    fn created_as_chrono(&self) -> Result<Option<DateTime<FixedOffset>>, io::Error>;

    fn created_as_chrono_or_now(&self) -> Result<DateTime<FixedOffset>, io::Error> {
        let time = self
            .created_as_chrono()?
            .unwrap_or_else(|| Local::now().into());
        Ok(time)
    }

    /// Get the modification time of the file as a DateTime<FixedOffset>.
    fn modified_as_chrono(&self) -> Result<Option<DateTime<FixedOffset>>, io::Error>;

    fn modified_as_chrono_or_now(&self) -> Result<DateTime<FixedOffset>, io::Error> {
        let time = self
            .modified_as_chrono()?
            .unwrap_or_else(|| Local::now().into());
        Ok(time)
    }
}
impl MetadataUtils for std::fs::Metadata {
    fn created_as_chrono(&self) -> Result<Option<DateTime<FixedOffset>>, io::Error> {
        self.created()
            .ok()
            .map(system_time_to_date_time)
            .transpose()
    }
    fn modified_as_chrono(&self) -> Result<Option<DateTime<FixedOffset>>, io::Error> {
        self.modified()
            .ok()
            .map(system_time_to_date_time)
            .transpose()
    }
}
