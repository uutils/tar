// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use std::collections::VecDeque;
use std::fs::{self, File};
use std::path::Component::{self, ParentDir, RootDir};
use std::path::{self, Path, PathBuf};
use tar::Builder;
use uucore::error::UResult;

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
pub fn create_archive(archive_path: &Path, files: &[&Path], verbose: bool) -> UResult<()> {
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
    for &path in files {
        // Check if path exists
        if !path.exists() {
            return Err(TarError::FileNotFound(path.display().to_string()).into());
        }
      
        if verbose {
            let to_print = get_tree(path)?
                .iter()
                .map(|p| (p.is_dir(), p.display().to_string()))
                .map(|(is_dir, path)| {
                    if is_dir {
                        format!("{}{}", path, path::MAIN_SEPARATOR)
                    } else {
                        path
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            println!("{to_print}");
        }

        // Normalize path if needed (so far, handles only absolute paths)
        let normalized_name = if let Some(normalized) = normalize_path(path) {
            let original_components: Vec<Component> = path.components().collect();
            let normalized_components: Vec<Component> = normalized.components().collect();
            if original_components.len() > normalized_components.len() {
                let removed: PathBuf = original_components
                    [..original_components.len() - normalized_components.len()]
                    .iter()
                    .collect();
                println!("Removing leading `{}' from member names", removed.display());
            }

            normalized
        } else {
            path.to_path_buf()
        };

        // If it's a directory, recursively add all contents
        if path.is_dir() {
            builder.append_dir_all(normalized_name, path).map_err(|e| {
                TarError::TarOperationError(format!(
                    "Failed to add directory '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        } else {
            // For files, add them directly
            builder
                .append_path_with_name(path, normalized_name)
                .map_err(|e| {
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

fn get_tree(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = Vec::new();
    let mut stack = VecDeque::new();
    stack.push_back(path.to_path_buf());

    while let Some(current) = stack.pop_back() {
        paths.push(current.clone());
        if current.is_dir() {
            for entry in fs::read_dir(current)? {
                let child = entry?.path();
                stack.push_back(child);
            }
        }
    }

    Ok(paths)
}

fn normalize_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(
            path.components()
                .filter(|c| !matches!(c, RootDir | ParentDir))
                .collect::<PathBuf>(),
        )
    } else {
        None
    }
}
