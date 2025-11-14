// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::operation::TarOperation;
use crate::options::options::{TarOption, TarParams};
use std::fs::File;
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;

pub(crate) struct Extract;

impl TarOperation for Extract {
    fn exec(&self, options: &TarParams) -> UResult<()> {
        extract_archive(
            options.archive(),
            options
                .options()
                .iter()
                .any(|x| matches!(x, TarOption::Verbose)),
        )
    }
}

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
pub fn extract_archive(archive_path: &Path, verbose: bool) -> UResult<()> {
    // Open the archive file
    let file = File::open(archive_path).map_err(|e| {
        TarError::TarOperationError(format!(
            "Cannot open archive '{}': {}",
            archive_path.display(),
            e
        ))
    })?;

    // Create Archive instance
    let mut archive = Archive::new(file);

    // Extract to current directory
    if verbose {
        println!("Extracting archive: {}", archive_path.display());
    }

    // Iterate through entries for verbose output and error handling
    for entry_result in archive
        .entries()
        .map_err(|e| TarError::InvalidArchive(format!("Failed to read archive entries: {e}")))?
    {
        let mut entry = entry_result
            .map_err(|e| TarError::InvalidArchive(format!("Failed to read entry: {e}")))?;

        // Get the path before unpacking (clone it so we can use it after borrowing entry mutably)
        let path = entry
            .path()
            .map_err(|e| TarError::InvalidArchive(format!("Failed to read entry path: {e}")))?
            .to_path_buf();

        if verbose {
            println!("{}", path.display());
        }

        // Unpack the entry
        entry.unpack_in(".").map_err(|e| {
            TarError::TarOperationError(format!("Failed to extract '{}': {}", path.display(), e))
        })?;
    }

    Ok(())
}
