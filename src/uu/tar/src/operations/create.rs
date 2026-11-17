// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::ArchiveWriter;
use crate::errors::{finish_after_previous_errors, report_recoverable, TarError};
use crate::CompressionMode;
use std::collections::VecDeque;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Component::{self, ParentDir, Prefix, RootDir};
use std::path::{self, Path, PathBuf};
use tar::Builder;
use uucore::error::UResult;

/// Create a tar archive from the specified files
///
/// # Arguments
///
/// * `output` - Destination where the tar archive should be written
/// * `files` - Slice of file paths to add to the archive
/// * `allow_absolute` - Allow absolute paths while creating archive
/// * `verbose` - Whether to print verbose output during creation
/// * `recursive` - Whether to add the contents of directories, not only the directories themselves
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be created
/// - Any input file cannot be read (a missing input file is reported and
///   skipped, and the archive is still written)
/// - Files cannot be added due to I/O or permission errors
pub fn create_archive(
    output: impl Write,
    status_output: impl Write,
    files: &[&Path],
    allow_absolute: bool,
    verbose: bool,
    recursive: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let output = BufWriter::new(output);
    let mut status_output = BufWriter::new(status_output);

    // Create Builder instance
    let writer = ArchiveWriter::new(output, compression)?;
    let mut builder = Builder::new(writer);
    builder.preserve_absolute(allow_absolute);
    // Like GNU tar without -h/--dereference, archive symlinks as symlinks,
    // both for operands and for entries found inside directories.
    builder.follow_symlinks(false);

    // Set when an operand could not be archived but the rest were, so that tar
    // ends with "Exiting with failure status due to previous errors" like GNU tar.
    let mut missing_operand = false;

    // Add each file or directory to the archive
    for &path in files {
        // A missing operand is reported and skipped; the other operands are
        // still archived, as with GNU tar. `symlink_metadata` does not follow
        // symlinks, so a dangling symlink is not treated as missing.
        let metadata = match path.symlink_metadata() {
            Ok(metadata) => metadata,
            Err(source) => {
                status_output.flush().map_err(TarError::Io)?;
                report_recoverable(&TarError::CannotStat {
                    path: path.to_path_buf(),
                    source,
                })?;
                missing_operand = true;
                continue;
            }
        };

        if verbose {
            print_verbose_tree(&mut status_output, path, recursive)?;
        }

        let normalized_name = get_normalized_path(path, allow_absolute)?;

        if metadata.is_dir() {
            if recursive {
                // If it's a directory, recursively add all contents
                builder.append_dir_all(normalized_name, path)
            } else {
                // --no-recursion: the directory entry only, named with a trailing
                // slash like the top directory added by append_dir_all
                builder.append_dir(normalized_name.join(""), path)
            }
            .map_err(|e| TarError::CannotAddDirectory {
                path: path.to_path_buf(),
                source: e,
            })?;
        } else {
            // For files and symlinks, add them directly
            builder
                .append_path_with_name(path, normalized_name)
                .map_err(|e| TarError::CannotAddFile {
                    path: path.to_path_buf(),
                    source: e,
                })?;
        }
    }

    builder.finish().map_err(TarError::CannotFinalizeArchive)?;
    let writer = builder
        .into_inner()
        .map_err(|e| TarError::TarOperationError(format!("Failed to finalize archive: {e}")))?;
    writer.finish()?;
    status_output.flush().map_err(TarError::Io)?;

    if missing_operand {
        return finish_after_previous_errors();
    }
    Ok(())
}

fn print_verbose_tree(status_output: &mut impl Write, path: &Path, recursive: bool) -> UResult<()> {
    let tree = if recursive {
        get_tree(path)?
    } else {
        vec![path.to_path_buf()]
    };
    let to_print = tree
        .iter()
        .map(|p| (is_real_dir(p), p.display().to_string()))
        .map(|(is_dir, path)| {
            if is_dir {
                format!("{}{}", path, path::MAIN_SEPARATOR)
            } else {
                path
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    writeln!(status_output, "{to_print}").map_err(TarError::Io)?;
    Ok(())
}

fn get_normalized_path(path: &Path, allow_absolute: bool) -> Result<PathBuf, TarError> {
    if let Some(normalized) = normalize_path(path, allow_absolute) {
        let original_components: Vec<Component> = path.components().collect();
        let normalized_components: Vec<Component> = normalized.components().collect();
        if original_components.len() > normalized_components.len() {
            let removed: PathBuf = original_components
                [..original_components.len() - normalized_components.len()]
                .iter()
                .collect();
            super::warn_removed_prefix(removed.display())?;
        }
        Ok(normalized)
    } else {
        Ok(path.to_path_buf())
    }
}

/// Returns true for a directory, but not for a symlink to a directory, which
/// is archived as a symlink.
fn is_real_dir(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|m| m.is_dir())
}

fn get_tree(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = Vec::new();
    let mut stack = VecDeque::new();
    stack.push_back(path.to_path_buf());

    while let Some(current) = stack.pop_back() {
        paths.push(current.clone());
        if is_real_dir(&current) {
            for entry in fs::read_dir(current)? {
                let child = entry?.path();
                stack.push_back(child);
            }
        }
    }

    Ok(paths)
}

fn normalize_path(path: &Path, allow_absolute: bool) -> Option<PathBuf> {
    if path.is_absolute() && !allow_absolute {
        Some(
            path.components()
                .filter(|c| !matches!(c, RootDir | ParentDir | Prefix(_)))
                .collect::<PathBuf>(),
        )
    } else {
        None
    }
}

#[cfg(test)]
#[path = "create_tests.rs"]
mod tests;
