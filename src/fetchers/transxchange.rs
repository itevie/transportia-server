use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

use anyhow::{Context, Result};
use walkdir::WalkDir;
use zip::ZipArchive;

/// Recursively finds every XML file in `source`.
///
/// - Normal XML files are opened directly.
/// - ZIP files are opened without extracting them.
/// - ZIPs inside ZIPs are recursively processed.
/// - Each XML is passed to `callback` one at a time.
///
/// The XML data is only kept alive for the duration of the callback.
pub fn for_each_xml<F>(source: &Path, mut callback: F) -> Result<()>
where
    F: FnMut(&str, &[u8]) -> Result<()>,
{
    process_path(source, &mut callback)
}

fn process_path<F>(path: &Path, callback: &mut F) -> Result<()>
where
    F: FnMut(&str, &[u8]) -> Result<()>,
{
    if path.is_dir() {
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            let entry_path = entry.path();

            if entry_path.is_file() {
                process_path(entry_path, callback)?;
            }
        }

        return Ok(());
    }

    match extension(path).as_deref() {
        Some("xml") => {
            let data = std::fs::read(path)
                .with_context(|| format!("Failed to read XML: {}", path.display()))?;

            callback(&path.to_string_lossy(), &data)?;
        }

        Some("zip") => {
            process_zip(path, callback)?;
        }

        _ => {}
    }

    Ok(())
}

fn process_zip<F>(path: &Path, callback: &mut F) -> Result<()>
where
    F: FnMut(&str, &[u8]) -> Result<()>,
{
    let file =
        File::open(path).with_context(|| format!("Failed to open ZIP: {}", path.display()))?;

    let mut archive =
        ZipArchive::new(file).with_context(|| format!("Failed to read ZIP: {}", path.display()))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .with_context(|| format!("Failed to read entry {} from {}", i, path.display()))?;

        if entry.is_dir() {
            continue;
        }

        // Make the name owned so `entry` can be mutably borrowed later.
        let name = entry
            .name()
            .with_context(|| format!("Failed to get entry name in {}", path.display()))?
            .to_string();

        if name.to_ascii_lowercase().ends_with(".xml") {
            let mut data = Vec::new();

            entry
                .read_to_end(&mut data)
                .with_context(|| format!("Failed to extract {} from {}", name, path.display()))?;

            let display_name = format!("{} -> {}", path.display(), name);

            callback(&display_name, &data)?;
        } else if name.to_ascii_lowercase().ends_with(".zip") {
            let mut data = Vec::new();

            entry.read_to_end(&mut data).with_context(|| {
                format!(
                    "Failed to extract nested ZIP {} from {}",
                    name,
                    path.display()
                )
            })?;

            let display_name = format!("{} -> {}", path.display(), name);

            // Nested ZIP only exists in memory.
            process_zip_bytes(&display_name, &data, callback)?;
        }
    }

    Ok(())
}

fn process_zip_bytes<F>(name: &str, data: &[u8], callback: &mut F) -> Result<()>
where
    F: FnMut(&str, &[u8]) -> Result<()>,
{
    let cursor = Cursor::new(data);

    let mut archive =
        ZipArchive::new(cursor).with_context(|| format!("Failed to read nested ZIP: {}", name))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .with_context(|| format!("Failed to read entry {} from {}", i, name))?;

        if entry.is_dir() {
            continue;
        }

        let entry_name = entry
            .name()
            .with_context(|| format!("Failed to get entry name in {}", name))?;

        let child_name = format!("{} -> {}", name, entry_name.as_ref());

        if entry_name.as_ref().to_ascii_lowercase().ends_with(".xml") {
            let mut xml = Vec::new();

            entry.read_to_end(&mut xml)?;

            callback(&child_name, &xml)?;
        } else if entry_name.as_ref().to_ascii_lowercase().ends_with(".zip") {
            let mut nested_zip = Vec::new();

            entry.read_to_end(&mut nested_zip)?;

            process_zip_bytes(&child_name, &nested_zip, callback)?;
        }
    }

    Ok(())
}

// fn display_name(zip: &Path, entry: &str) -> String {
//     format!("{} -> {}", zip.display(), entry)
// }

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_ascii_lowercase())
}
