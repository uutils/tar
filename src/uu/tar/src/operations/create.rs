// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::TarOperation;
use crate::options::{TarOption, TarParams};
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use tar::Builder;
use uucore::error::UResult;

pub struct Create;

impl TarOperation for Create {
    fn exec(&self, options: &TarParams) -> UResult<()> {
        create_archive(
            options.archive(),
            options.files().as_slice(),
            options
                .options()
                .iter()
                .any(|x| matches!(x, TarOption::Verbose)),
        )
    }
}

/// Create a tar archive from the specified files
///
/// # Arguments
///
/// * `archive_path` - Path where the tar archive should be created
/// * `files` - Slice of file paths to add to the archive
/// * `verbose` - Whether to print verbose output during creation
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be created
/// - Any input file cannot be read
/// - Files cannot be added due to I/O or permission errors
pub fn create_archive(archive_path: &Path, files: &[PathBuf], verbose: bool) -> UResult<()> {
    // Create the output file
    let file = File::create(archive_path).map_err(|e| {
        TarError::TarOperationError(format!(
            "Cannot create archive '{}': {}",
            archive_path.display(),
            e
        ))
    })?;

    // Create Builder instance
    let mut builder = Builder::new(file);

    if verbose {
        println!("Creating archive: {}", archive_path.display());
    }

    // Add each file or directory to the archive
    for path in files {
        if verbose {
            println!("{}", path.display());
        }

        // Check if path exists
        if !path.exists() {
            return Err(TarError::FileNotFound(path.display().to_string()).into());
        }

        // If it's a directory, recursively add all contents
        if path.is_dir() {
            builder.append_dir_all(path, path).map_err(|e| {
                TarError::TarOperationError(format!(
                    "Failed to add directory '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        } else {
            // For files, add them directly
            builder.append_path(path).map_err(|e| {
                TarError::TarOperationError(format!(
                    "Failed to add file '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        }
    }

    // Finish writing the archive
    builder
        .finish()
        .map_err(|e| TarError::TarOperationError(format!("Failed to finalize archive: {e}")))?;

    Ok(())
}
