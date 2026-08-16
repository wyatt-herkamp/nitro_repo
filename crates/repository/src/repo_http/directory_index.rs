//! HTML index for a repository directory.
//!
//! `GET` on a directory used to answer `501 Not Implemented` with the literal body
//! `"Build HTML Page listing"`. That is not just cosmetic: Maven walks directory listings to
//! resolve version ranges and `LATEST`/`RELEASE`, and anyone browsing a repository in a browser
//! lands here. The output deliberately follows the same shape as every other repository manager's
//! index (a table of links, parent directory first), because that is what tooling expects to parse.
use std::fmt::Write;

use nr_core::storage::StoragePath;
use nr_storage::{DirectoryFileType, FileType, StorageFileMeta};

/// Renders the listing for one directory.
///
/// `path` is the request path the listing was reached by, used to decide whether a parent link is
/// meaningful. Entry links are relative, so the caller redirects a directory URL that lacks a
/// trailing slash before rendering — without it a client resolves `child` against the *parent*.
pub fn render(
    path: &StoragePath,
    meta: &StorageFileMeta<DirectoryFileType>,
    files: &[StorageFileMeta<FileType>],
) -> String {
    let title = if path.number_of_components() == 0 {
        "/".to_owned()
    } else {
        format!("/{path}")
    };

    let mut entries: Vec<&StorageFileMeta<FileType>> = files.iter().collect();
    // Directories first, then by name — the ordering every other index uses, and the one that
    // makes a long listing navigable.
    entries.sort_by(|left, right| {
        let left_is_dir = matches!(left.file_type, FileType::Directory(_));
        let right_is_dir = matches!(right.file_type, FileType::Directory(_));
        right_is_dir
            .cmp(&left_is_dir)
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut body = String::with_capacity(512 + entries.len() * 128);
    let _ = write!(
        body,
        concat!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">",
            "<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">",
            "<title>Index of {title}</title><style>",
            "body{{font-family:system-ui,sans-serif;margin:2rem;}}",
            "table{{border-collapse:collapse;min-width:min(40rem,100%);}}",
            "th,td{{text-align:left;padding:.35rem .75rem;}}",
            "th{{border-bottom:1px solid currentColor;}}",
            "td.size{{text-align:right;font-variant-numeric:tabular-nums;}}",
            "</style></head><body><h1>Index of {title}</h1>",
            "<table><thead><tr><th>Name</th><th>Last modified</th>",
            "<th class=\"size\">Size</th></tr></thead><tbody>"
        ),
        title = Escaped(&title)
    );

    if path.number_of_components() > 0 {
        body.push_str("<tr><td><a href=\"../\">../</a></td><td></td><td class=\"size\"></td></tr>");
    }

    for entry in &entries {
        let is_directory = matches!(entry.file_type, FileType::Directory(_));
        // A directory link needs its own trailing slash for the same reason the listing URL does.
        let suffix = if is_directory { "/" } else { "" };
        let size = match &entry.file_type {
            FileType::File(file) => human_size(file.file_size),
            FileType::Directory(directory) => format!("{} items", directory.file_count),
        };
        let _ = write!(
            body,
            "<tr><td><a href=\"{href}{suffix}\">{name}{suffix}</a></td>\
             <td>{modified}</td><td class=\"size\">{size}</td></tr>",
            href = Escaped(&entry.name),
            name = Escaped(&entry.name),
            modified = entry.modified.format("%Y-%m-%d %H:%M:%S %Z"),
            size = Escaped(&size),
        );
    }

    let _ = write!(
        body,
        "</tbody></table><p>{count} entries</p></body></html>",
        count = meta.file_type.file_count.max(entries.len() as u64)
    );
    body
}

/// Renders a value with HTML metacharacters escaped.
///
/// Entry names come from whatever a client uploaded, so they reach this page as untrusted text.
struct Escaped<'a>(&'a str);
impl std::fmt::Display for Escaped<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for character in self.0.chars() {
            match character {
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                '"' => f.write_str("&quot;")?,
                '\'' => f.write_str("&#39;")?,
                other => f.write_char(other)?,
            }
        }
        Ok(())
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use nr_core::storage::FileHashes;
    use nr_storage::FileFileType;

    use super::*;

    fn now() -> chrono::DateTime<chrono::FixedOffset> {
        Local::now().fixed_offset()
    }
    fn file(name: &str, size: u64) -> StorageFileMeta<FileType> {
        StorageFileMeta::new(
            name,
            FileType::File(FileFileType {
                file_size: size,
                mime_type: None,
                file_hash: FileHashes::default(),
            }),
            now(),
            now(),
        )
    }
    fn directory(name: &str) -> StorageFileMeta<FileType> {
        StorageFileMeta::new(
            name,
            FileType::Directory(DirectoryFileType { file_count: 2 }),
            now(),
            now(),
        )
    }
    fn directory_meta(count: u64) -> StorageFileMeta<DirectoryFileType> {
        StorageFileMeta::new("dir", DirectoryFileType { file_count: count }, now(), now())
    }

    #[test]
    fn lists_every_entry_with_a_link() {
        let files = vec![file("app-1.0.0.jar", 2048), directory("1.0.0")];
        let html = render(
            &StoragePath::from("dev/kingtux/app/"),
            &directory_meta(2),
            &files,
        );

        assert!(html.contains("href=\"app-1.0.0.jar\""));
        // A directory link needs its own trailing slash or the next request resolves one level up.
        assert!(html.contains("href=\"1.0.0/\""));
    }

    /// Maven follows `../` up a tree, and a root listing has nowhere to go.
    #[test]
    fn parent_link_only_below_the_root() {
        let nested = render(&StoragePath::from("dev/kingtux/"), &directory_meta(0), &[]);
        assert!(nested.contains("href=\"../\""));

        let root = render(&StoragePath::default(), &directory_meta(0), &[]);
        assert!(!root.contains("href=\"../\""));
    }

    #[test]
    fn directories_sort_before_files() {
        let files = vec![file("aaa.jar", 1), directory("zzz")];
        let html = render(&StoragePath::from("x/"), &directory_meta(2), &files);

        let directory_at = html.find("zzz/").expect("directory missing");
        let file_at = html.find("aaa.jar").expect("file missing");
        assert!(directory_at < file_at, "files sorted before directories");
    }

    /// Entry names are whatever a client uploaded, so they are untrusted text on this page.
    #[test]
    fn entry_names_are_escaped() {
        let files = vec![file("<script>alert(1)</script>", 1)];
        let html = render(&StoragePath::from("x/"), &directory_meta(1), &files);

        assert!(
            !html.contains("<script>alert(1)</script>"),
            "an uploaded name was rendered as markup: {html}"
        );
        assert!(html.contains("&lt;script&gt;"));
    }
}
