// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::{finish_after_previous_errors, report_recoverable, TarError};
use crate::CompressionMode;
use filetime::FileTime;
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use tar::{Archive, Entry, EntryType};
use uucore::error::UResult;

/// Extract files from a tar archive
///
/// # Arguments
///
/// * `archive_path` - Path to the tar archive to extract
/// * `verbose` - Whether to print verbose output during extraction
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be opened
/// - The archive format is invalid
/// - Files cannot be extracted due to I/O or permission errors
pub fn extract_archive(
    input: impl Read,
    archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(input, compression)?;
    let mut archive = Archive::new(reader);
    let mut out = BufWriter::new(io::stdout().lock());

    // Extract to current directory
    if verbose {
        writeln!(out, "Extracting archive: {}", archive_path.display()).map_err(TarError::Io)?;
    }

    // Members that were refused are reported as they are met; like GNU tar, the
    // remaining members are still extracted and the exit status records the failure.
    let mut refused_member = false;

    // Like GNU tar, directories get their permissions after their contents
    // are extracted, or a read-only directory could not be filled.
    let mut directories = Vec::new();

    // Iterate through entries for verbose output and error handling
    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let mut entry = entry_result.map_err(TarError::CannotReadEntry)?;

        // Get the path before unpacking (clone it so we can use it after borrowing entry mutably)
        let path = entry
            .path()
            .map_err(TarError::CannotReadEntryPath)?
            .to_path_buf();

        if verbose {
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }

        // Directories with a `..` component are not deferred, so they are
        // refused in archive order
        if entry.header().entry_type() == EntryType::Directory
            && !path.components().any(|c| c == Component::ParentDir)
        {
            directories.push((path, entry));
            continue;
        }
        refused_member |= unpack_entry(&mut entry, path, &mut out)?;
    }

    // Children before parents, so a read-only parent is restored last
    directories.sort_by(|a, b| b.0.cmp(&a.0));
    for (path, mut entry) in directories {
        let mtime = entry.header().mtime().ok();
        let refused = unpack_entry(&mut entry, path.clone(), &mut out)?;
        refused_member |= refused;
        // The archive's mtime, as GNU tar restores it (tar-rs only does it for files)
        if let (false, Some(mtime)) = (refused, mtime) {
            if let Err(e) = set_directory_mtime(&path, mtime) {
                out.flush().map_err(TarError::Io)?;
                report_recoverable(&TarError::CannotUtime { path, source: e })?;
                refused_member = true;
            }
        }
    }

    out.flush().map_err(TarError::Io)?;
    if refused_member {
        return finish_after_previous_errors();
    }
    Ok(())
}

/// Unpacks one entry into the current directory. Returns true when the entry
/// was refused, which has already been reported.
fn unpack_entry<R: Read>(
    entry: &mut Entry<R>,
    path: PathBuf,
    out: &mut impl Write,
) -> UResult<bool> {
    // `unpack_in` skips, and returns false for, a member whose name contains
    // a `..` component, as it would escape the target directory.
    let unpacked = entry.unpack_in(".").map_err(|e| TarError::CannotExtract {
        path: path.clone(),
        source: e,
    })?;
    if !unpacked {
        out.flush().map_err(TarError::Io)?;
        report_recoverable(&TarError::MemberNameContainsDotDot { path })?;
    }
    Ok(!unpacked)
}

/// Sets the mtime of the directory `unpack_in` extracted for the member `path`.
fn set_directory_mtime(path: &Path, mtime: u64) -> io::Result<()> {
    // `unpack_in` drops the root, `.` and prefix components of member names
    let dst: PathBuf = path
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .collect();
    let dst = if dst.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        dst
    };
    // Negative times come back from tar-rs as large u64 values; cast like it does
    filetime::set_file_mtime(&dst, FileTime::from_unix_time(mtime as i64, 0))
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
