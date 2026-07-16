// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::ArchiveWriter;
use crate::errors::TarError;
use crate::CompressionMode;
use std::collections::VecDeque;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Component::{self, ParentDir, Prefix, RootDir};
use std::path::{self, Path, PathBuf};
use tar::Builder;
use uucore::error::UResult;

/// Create a tar archive from the specified files.
///
/// # Arguments
///
/// * `output` - Destination where the tar archive data should be written.
/// * `status_output` - Writer for verbose progress output (e.g. stderr/stdout).
/// * `files` - Slice of file paths to add to the archive.
/// * `allow_absolute` - Allow absolute paths while creating archive.
/// * `verbose` - Whether to print verbose output during creation.
/// * `compression` - The compression mode to apply to the output.
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be created
/// - Any input file cannot be read
/// - Files cannot be added due to I/O or permission errors
pub fn create_archive(
    output: impl Write,
    status_output: impl Write,
    files: &[&Path],
    allow_absolute: bool,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let output = BufWriter::new(output);
    let mut status_output = BufWriter::new(status_output);

    // Create Builder instance
    let writer = ArchiveWriter::new(output, compression)?;
    let mut builder = Builder::new(writer);
    builder.preserve_absolute(allow_absolute);
    builder.follow_symlinks(false);

    // Add each file or directory to the archive
    for &path in files {
        // Check if path exists (without following symlinks)
        let metadata = match path.symlink_metadata() {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(TarError::FileNotFound {
                    path: path.to_path_buf(),
                }
                .into());
            }
            Err(e) => return Err(TarError::Io(e).into()),
        };

        if verbose {
            print_verbose_tree(&mut status_output, path)?;
        }

        let normalized_name = get_normalized_path(path, allow_absolute)?;

        // If it's a directory, recursively add all contents
        if metadata.is_dir() {
            builder.append_dir_all(normalized_name, path).map_err(|e| {
                TarError::CannotAddDirectory {
                    path: path.to_path_buf(),
                    source: e,
                }
            })?;
        } else {
            // For files/symlinks, add them directly
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

    Ok(())
}

/// Prints the list of files in `path` recursively in a verbose format.
///
/// Traverses the directory tree at `path` and writes each entry name to
/// `status_output`. Directory entries are printed with a trailing slash.
/// Does not follow symlinks.
fn print_verbose_tree(status_output: &mut impl Write, path: &Path) -> UResult<()> {
    for (p, is_dir) in get_tree(path)? {
        if is_dir {
            writeln!(status_output, "{}{}", p.display(), path::MAIN_SEPARATOR)
                .map_err(TarError::Io)?;
        } else {
            writeln!(status_output, "{}", p.display()).map_err(TarError::Io)?;
        }
    }
    Ok(())
}

fn needs_cleaning(path: &Path, allow_absolute: bool) -> bool {
    for component in path.components() {
        match component {
            Prefix(_) | RootDir => {
                if !allow_absolute {
                    return true;
                }
            }
            Component::CurDir | ParentDir => return true,
            Component::Normal(_) => {}
        }
    }
    false
}

/// Normalizes the path for archiving and displays a warning if leading components are stripped.
///
/// Lexically cleans the path first (resolving `.` and `..`). If `allow_absolute`
/// is false, it strips leading absolute prefixes (`RootDir`, `Prefix`) and
/// leading `ParentDir` (`..`) components. Prints a warning to stderr if any
/// prefix components were removed.
fn get_normalized_path(path: &Path, allow_absolute: bool) -> Result<PathBuf, TarError> {
    if !needs_cleaning(path, allow_absolute) {
        return Ok(path.to_path_buf());
    }
    let cleaned = clean_path(path);
    let mut normalized = cleaned.clone();
    let mut prefix_removed = PathBuf::new();
    let mut changed = cleaned != path;

    if !allow_absolute {
        let mut remaining = PathBuf::new();
        let mut stripped_any = false;
        let mut in_leading = true;
        for c in cleaned.components() {
            if in_leading && matches!(c, RootDir | Prefix(_) | ParentDir) {
                stripped_any = true;
                prefix_removed.push(c.as_os_str());
            } else {
                in_leading = false;
                remaining.push(c.as_os_str());
            }
        }
        if stripped_any {
            normalized = remaining;
            changed = true;
        }
    }

    if changed {
        if !prefix_removed.as_os_str().is_empty() {
            writeln!(
                std::io::stderr(),
                "tar: Removing leading `{}' from member names",
                prefix_removed.display()
            )?;
        }
        Ok(normalized)
    } else {
        Ok(path.to_path_buf())
    }
}

/// Recursively gathers all paths within `path` without following symlinks.
///
/// Returns a list of tuples containing the path and a boolean indicating
/// whether the path is a directory. Using `symlink_metadata` and
/// `DirEntry::file_type` prevents recursing into symlinked directories.
fn get_tree(path: &Path) -> Result<Vec<(PathBuf, bool)>, std::io::Error> {
    let mut paths = Vec::new();
    let mut stack = VecDeque::new();

    let root_metadata = path.symlink_metadata()?;
    let root_is_dir = root_metadata.is_dir();
    stack.push_back((path.to_path_buf(), root_is_dir));

    while let Some((current, is_dir)) = stack.pop_back() {
        if is_dir {
            for entry in fs::read_dir(&current)? {
                let entry = entry?;
                let child = entry.path();
                let file_type = entry.file_type()?;
                let child_is_dir = file_type.is_dir();
                stack.push_back((child, child_is_dir));
            }
        }
        paths.push((current, is_dir));
    }

    Ok(paths)
}

/// Performs lexical cleaning of a path.
///
/// Resolves `.` and `..` components where possible without accessing the
/// actual filesystem. This is used to normalize paths safely and prevent
/// equivalent path confusion.
fn clean_path(path: &Path) -> PathBuf {
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Prefix(_) | RootDir => {
                clean.push(component.as_os_str());
            }
            Component::CurDir => {}
            ParentDir => {
                if let Some(last) = clean.components().next_back() {
                    match last {
                        Component::Normal(_) => {
                            clean.pop();
                        }
                        RootDir | Prefix(_) => {
                            // Cannot go above root/prefix
                        }
                        ParentDir => {
                            clean.push(component);
                        }
                        Component::CurDir => unreachable!(),
                    }
                } else {
                    clean.push(component);
                }
            }
            Component::Normal(c) => {
                clean.push(c);
            }
        }
    }
    if clean.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        clean
    }
}

#[cfg(test)]
#[path = "create_tests.rs"]
mod tests;
