// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::TarError;
use crate::CompressionMode;
use chrono::{TimeZone, Utc};
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use uucore::error::UResult;
use uucore::fs::display_permissions_unix;

use super::{entry_matches, strip_leading_components};

/// List the contents of a tar archive, printing one entry per line.
///
/// # Arguments
///
/// * `input` - Readable source of the archive data
/// * `_archive_path` - Path used for error messages (reserved for future use)
/// * `verbose` - Print detailed metadata for each entry
/// * `compression` - Compression mode to use when reading
/// * `file_patterns` - If non-empty, only show entries matching these names (or globs)
/// * `wildcards` - Treat `file_patterns` as glob patterns (`*`, `?`)
/// * `strip_components` - Strip this many leading path components from displayed paths
///
/// # Errors
///
/// Returns an error if the archive cannot be read or an entry path cannot be decoded.
pub fn list_archive(
    input: impl Read,
    _archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
    file_patterns: &[PathBuf],
    wildcards: bool,
    strip_components: u32,
) -> UResult<()> {
    let reader = open_archive_reader(input, compression)?;
    let mut archive = Archive::new(reader);
    let mut out = BufWriter::new(io::stdout().lock());

    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let entry = entry_result.map_err(TarError::CannotReadEntry)?;

        let path = entry
            .path()
            .map_err(TarError::CannotReadEntryPath)?
            .to_path_buf();

        if !entry_matches(&path, file_patterns, wildcards) {
            continue;
        }

        let display_path = if strip_components > 0 {
            match strip_leading_components(&path, strip_components) {
                Some(p) => p,
                None => continue,
            }
        } else {
            path.clone()
        };

        if verbose {
            let formatted = format_verbose_entry(&entry, &display_path)?;
            writeln!(out, "{formatted}").map_err(TarError::Io)?;
        } else {
            writeln!(out, "{}", display_path.display()).map_err(TarError::Io)?;
        }
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}

fn format_verbose_entry<R: Read>(
    entry: &tar::Entry<'_, R>,
    display_path: &Path,
) -> Result<String, TarError> {
    let (mode, entry_type, owner, group, size, mtime) = {
        let header = entry.header();
        (
            header.mode().unwrap_or(0),
            header.entry_type(),
            header
                .username()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_owned(),
            header
                .groupname()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_owned(),
            header.size().unwrap_or(0),
            header.mtime().unwrap_or(0),
        )
    };

    let type_char = match entry_type {
        tar::EntryType::Directory => 'd',
        tar::EntryType::Symlink => 'l',
        tar::EntryType::Char => 'c',
        tar::EntryType::Block => 'b',
        tar::EntryType::Fifo => 'p',
        _ => '-',
    };
    let perm_str = display_permissions_unix(mode, false);
    let permissions = format!("{type_char}{perm_str}");

    let dt: chrono::DateTime<Utc> = Utc
        .timestamp_opt(mtime as i64, 0)
        .single()
        .unwrap_or_else(Utc::now);
    let date_str = dt.format("%Y-%m-%d %H:%M");

    Ok(format!(
        "{permissions} {owner}/{group} {size:>8} {date_str} {}",
        display_path.display()
    ))
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
