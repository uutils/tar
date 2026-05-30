// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::display;
use crate::errors::TarError;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;

/// Extract files from a tar archive
///
/// # Arguments
///
/// * `archive_path` - Path to the tar archive to extract
/// * `verbose` - Verbosity level during extraction
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be opened
/// - The archive format is invalid
/// - Files cannot be extracted due to I/O or permission errors
pub fn extract_archive(archive_path: &Path, verbose: u8) -> UResult<()> {
    // Open the archive file
    let file = File::open(archive_path).map_err(|e| TarError::from_io_error(e, archive_path))?;

    // Create Archive instance
    let mut archive = Archive::new(file);
    let mut out = BufWriter::new(io::stdout().lock());

    // Extract to current directory
    if verbose >= 1 {
        writeln!(out, "Extracting archive: {}", archive_path.display()).map_err(TarError::Io)?;
    }

    // Iterate through entries for verbose output and error handling
    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let mut entry = entry_result.map_err(TarError::CannotReadEntry)?;

        // Get the path before unpacking (clone it so we can use it after borrowing entry mutably)
        let path = entry
            .path()
            .map_err(TarError::CannotReadEntryPath)?
            .to_path_buf();

        if verbose >= 2 {
            display::print_entry_verbose(&mut out, entry.header(), &path).map_err(TarError::Io)?;
        } else if verbose == 1 {
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }

        // Unpack the entry
        entry.unpack_in(".").map_err(|e| TarError::CannotExtract {
            path: path.clone(),
            source: e,
        })?;
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}
