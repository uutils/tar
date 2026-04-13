// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::compression::ArchiveWriter;
use crate::CompressionMode;
use std::collections::VecDeque;
use std::fs;
use std::path::Component::{self, ParentDir, Prefix, RootDir};
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
pub fn create_archive(
    archive_path: &Path,
    files: &[&Path],
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let writer = ArchiveWriter::create(archive_path, compression)?;
    let mut builder = Builder::new(writer);

    // Add each file or directory to the archive
    for &path in files {
        // Check if path exists
        if !path.exists() {
            return Err(TarError::FileNotFound {
                path: path.to_path_buf(),
            }
            .into());
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
                TarError::CannotAddDirectory {
                    path: path.to_path_buf(),
                    source: e,
                }
            })?;
        } else {
            // For files, add them directly
            builder
                .append_path_with_name(path, normalized_name)
                .map_err(|e| TarError::CannotAddFile {
                    path: path.to_path_buf(),
                    source: e,
                })?;
        }
    }

    // Finish writing the archive
    let writer = builder
        .into_inner()
        .map_err(|e| TarError::TarOperationError(format!("Failed to finalize archive: {e}")))?;
    writer.finish()?;

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
                .filter(|c| !matches!(c, RootDir | ParentDir | Prefix(_)))
                .collect::<PathBuf>(),
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompressionMode;
    use std::fs;
    use tar::Archive;
    use tempfile::tempdir;

    #[test]
    fn test_create_archive_with_zstd() {
        let tempdir = tempdir().unwrap();
        let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
        fs::write("file.txt", "hello").unwrap();

        create_archive(
            Path::new("archive.tar.zst"),
            &[Path::new("file.txt")],
            false,
            CompressionMode::Zstd,
        )
        .unwrap();

        let decoder =
            zstd::stream::read::Decoder::new(fs::File::open("archive.tar.zst").unwrap()).unwrap();
        let mut archive = Archive::new(decoder);
        let mut entries = archive.entries().unwrap();
        let entry = entries.next().unwrap().unwrap();
        assert_eq!(entry.path().unwrap().to_str(), Some("file.txt"));
    }

    #[test]
    fn test_create_archive_missing_file_fails() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("archive.tar.zst");
        let missing_path = tempdir.path().join("missing.txt");

        let err = create_archive(
            &archive_path,
            &[missing_path.as_path()],
            false,
            CompressionMode::Zstd,
        )
        .unwrap_err();
        assert!(err.to_string().contains("missing.txt"));
    }
}
