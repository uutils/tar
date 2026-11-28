// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::ArchiveWriter;
use crate::errors::{finish_after_previous_errors, report_recoverable, TarError};
use crate::CompressionMode;
use std::collections::VecDeque;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Component::{self, ParentDir, Prefix, RootDir};
use std::path::{self, Path, PathBuf};
use tar::Builder;
use uucore::error::UResult;
use uucore::fs::FileInformation;

/// Options for [`create_archive`]
pub struct CreateOptions {
    /// Allow absolute paths while creating archive
    pub allow_absolute: bool,
    /// Whether to print verbose output during creation
    pub verbose: bool,
    /// Whether to add the contents of directories, not only the directories themselves
    pub recursive: bool,
    /// Whether to remove the archived files once the archive is written
    pub remove_files: bool,
    /// The archive file, which is never removed, if it is not standard output
    pub archive: Option<FileInformation>,
}

/// Create a tar archive from the specified files
///
/// # Arguments
///
/// * `output` - Destination where the tar archive should be written
/// * `files` - Slice of file paths to add to the archive
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be created
/// - Any input file cannot be read (a missing input file is reported and
///   skipped, and the archive is still written)
/// - Files cannot be added due to I/O or permission errors
///
/// A file that cannot be removed is reported, and the other ones are still
/// removed.
pub fn create_archive(
    output: impl Write,
    status_output: impl Write,
    files: &[&Path],
    options: &CreateOptions,
    compression: CompressionMode,
) -> UResult<()> {
    let CreateOptions {
        allow_absolute,
        verbose,
        recursive,
        remove_files,
        ref archive,
    } = *options;
    let output = BufWriter::new(output);
    let mut status_output = BufWriter::new(status_output);

    // Create Builder instance
    let writer = ArchiveWriter::new(output, compression)?;
    let mut builder = Builder::new(writer);
    builder.preserve_absolute(allow_absolute);
    // Like GNU tar without -h/--dereference, archive symlinks as symlinks,
    // both for operands and for entries found inside directories.
    builder.follow_symlinks(false);

    // Set when an operand could not be archived but the rest were, or a file
    // could not be removed, so that tar ends with "Exiting with failure status
    // due to previous errors" like GNU tar.
    let mut previous_errors = false;

    // With --remove-files, what was archived, each entry after the ones it
    // contains. Like GNU tar, nothing is removed before the whole archive is
    // written, so a file can still be archived again by a later operand.
    let mut to_remove = Vec::new();

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
                previous_errors = true;
                continue;
            }
        };

        // Listed before archiving, so that --remove-files does not remove
        // anything created meanwhile that was not archived.
        let tree = if (verbose || remove_files) && recursive {
            get_tree(path).map_err(|source| TarError::CannotAddDirectory {
                path: path.to_path_buf(),
                source,
            })?
        } else {
            vec![path.to_path_buf()]
        };

        if verbose {
            print_verbose_tree(&mut status_output, &tree)?;
        }

        let normalized_name = get_normalized_path(path, allow_absolute)?;

        if remove_files {
            to_remove.extend(tree.into_iter().rev());
        }

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

    for path in &to_remove {
        if !remove_archived(path, archive.as_ref())? {
            previous_errors = true;
        }
    }

    if previous_errors {
        return finish_after_previous_errors();
    }
    Ok(())
}

/// Removes an archived file or (empty) directory for --remove-files, unless
/// it is the archive itself. One that is already gone, such as a file given
/// twice, is skipped. Returns false when it could not be removed, which is
/// reported.
fn remove_archived(path: &Path, archive: Option<&FileInformation>) -> UResult<bool> {
    let Ok(metadata) = path.symlink_metadata() else {
        return Ok(true);
    };
    if metadata.is_file()
        && archive.is_some_and(|archive| {
            FileInformation::from_path(path, false).is_ok_and(|file| file == *archive)
        })
    {
        return Ok(true);
    }
    let error = if metadata.is_dir() {
        match fs::remove_dir(path) {
            Err(source) if source.kind() != io::ErrorKind::NotFound => TarError::CannotRmdir {
                path: path.to_path_buf(),
                source,
            },
            _ => return Ok(true),
        }
    } else {
        match fs::remove_file(path) {
            Err(source) if source.kind() != io::ErrorKind::NotFound => TarError::CannotUnlink {
                path: path.to_path_buf(),
                source,
            },
            _ => return Ok(true),
        }
    };
    report_recoverable(&error)?;
    Ok(false)
}

fn print_verbose_tree(status_output: &mut impl Write, tree: &[PathBuf]) -> UResult<()> {
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
