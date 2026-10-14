// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::{finish_after_previous_errors, report_recoverable, TarError};
use crate::CompressionMode;
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;

/// Extract files from a tar archive
///
/// # Arguments
///
/// * `archive_path` - Path to the tar archive to extract
/// * `verbose` - Whether to print verbose output during extraction
/// * `to_stdout` - Write the contents of regular files to standard output instead
///   of creating them; verbose output then goes to standard error
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
    to_stdout: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(input, compression)?;
    let mut archive = Archive::new(reader);
    // With --to-stdout the member contents own stdout, so status messages go to stderr.
    let status: Box<dyn Write> = if to_stdout {
        Box::new(io::stderr())
    } else {
        Box::new(io::stdout().lock())
    };
    let mut out = BufWriter::new(status);
    let mut data = to_stdout.then(|| BufWriter::new(io::stdout().lock()));

    // Extract to current directory
    if verbose {
        writeln!(out, "Extracting archive: {}", archive_path.display()).map_err(TarError::Io)?;
    }

    // Members that were refused are reported as they are met; like GNU tar, the
    // remaining members are still extracted and the exit status records the failure.
    let mut refused_member = false;

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

        if let Some(data) = data.as_mut() {
            // Like GNU tar, only regular files have contents to write.
            if entry.header().entry_type().is_file() {
                out.flush().map_err(TarError::Io)?;
                io::copy(&mut entry, data).map_err(|e| TarError::CannotExtract {
                    path: path.clone(),
                    source: e,
                })?;
                // Keep each member's contents next to its name under -v.
                data.flush().map_err(TarError::Io)?;
            }
            continue;
        }

        // Unpack the entry. `unpack_in` skips, and returns false for, a member whose
        // name contains a `..` component, as it would escape the target directory.
        let unpacked = entry.unpack_in(".").map_err(|e| TarError::CannotExtract {
            path: path.clone(),
            source: e,
        })?;
        if !unpacked {
            out.flush().map_err(TarError::Io)?;
            report_recoverable(&TarError::MemberNameContainsDotDot { path: path.clone() })?;
            refused_member = true;
        }
    }

    out.flush().map_err(TarError::Io)?;
    if let Some(data) = data.as_mut() {
        data.flush().map_err(TarError::Io)?;
    }
    if refused_member {
        return finish_after_previous_errors();
    }
    Ok(())
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
