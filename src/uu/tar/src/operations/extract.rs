// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::{finish_after_previous_errors, report_recoverable, TarError};
use crate::CompressionMode;
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
        refused_member |= unpack_entry(&mut entry, path, &mut out)?;
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

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
