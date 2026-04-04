// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::compression::ArchiveWriter;
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
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be created
/// - Any input file cannot be read
/// - Files cannot be added due to I/O or permission errors
pub(crate) fn create_archive(
    output: impl Write,
    status_output: impl Write,
    files: &[&Path],
    allow_absolute: bool,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let output = ArchiveWriter::new(output, compression)?;
    let mut output = BufWriter::new(output);
    let mut status_output = BufWriter::new(status_output);

    // Create Builder instance
    let mut builder = Builder::new(&mut output);
    builder.preserve_absolute(allow_absolute);

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
            writeln!(status_output, "{to_print}").map_err(TarError::Io)?;
        }

        // Normalize path if needed (so far, handles only absolute paths)
        let normalized_name = if let Some(normalized) = normalize_path(path, allow_absolute) {
            let original_components: Vec<Component> = path.components().collect();
            let normalized_components: Vec<Component> = normalized.components().collect();
            if original_components.len() > normalized_components.len() {
                let removed: PathBuf = original_components
                    [..original_components.len() - normalized_components.len()]
                    .iter()
                    .collect();
                writeln!(
                    std::io::stderr(),
                    "tar: Removing leading `{}' from member names",
                    removed.display()
                )
                .map_err(TarError::Io)?;
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

    builder.finish().map_err(TarError::CannotFinalizeArchive)?;
    drop(builder);

    status_output.flush().map_err(TarError::Io)?;
    output.flush().map_err(TarError::Io)?;
    let output = output
        .into_inner()
        .map_err(|e| TarError::Io(e.into_error()))?;
    output.finish()?;

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
mod tests {
    use super::*;
    use std::io::{self, Write};
    use tar::Archive;
    use tempfile::{tempdir, TempDir};

    struct FailFlushWriter;
    impl Write for FailFlushWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush failed"))
        }
    }

    #[test]
    fn test_create_archive_flush_failed() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello").unwrap();

        let output = FailFlushWriter;
        let status_output = io::sink();

        let res = create_archive(
            output,
            status_output,
            &[file_path.as_path()],
            false,
            false,
            CompressionMode::None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_create_archive_with_zstd() {
        let tempdir = tempdir().unwrap();
        let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
        fs::write("file.txt", "hello").unwrap();

        create_archive(
            fs::File::create("archive.tar.zst").unwrap(),
            io::sink(),
            &[Path::new("file.txt")],
            false,
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
        let missing_path = tempdir.path().join("missing.txt");

        let err = create_archive(
            io::sink(),
            io::sink(),
            &[missing_path.as_path()],
            false,
            false,
            CompressionMode::Zstd,
        )
        .unwrap_err();
        assert!(err.to_string().contains("missing.txt"));
    }
}
