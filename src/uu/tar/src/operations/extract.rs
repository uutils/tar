// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::TarError;
use crate::CompressionMode;
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use tar::Archive;
use uucore::error::UResult;

use super::{entry_matches, strip_leading_components};

/// Extract files from a tar archive.
///
/// # Arguments
///
/// * `input` - Readable source of the archive data
/// * `archive_path` - Path used for error messages and verbose header
/// * `verbose` - Print each extracted path to stdout
/// * `compression` - Compression mode to use when reading
/// * `file_patterns` - If non-empty, only extract entries matching these names (or globs)
/// * `wildcards` - Treat `file_patterns` as glob patterns (`*`, `?`)
/// * `strip_components` - Strip this many leading path components before writing
///
/// # Errors
///
/// Returns an error if the archive cannot be read or entries cannot be extracted.
pub fn extract_archive(
    input: impl Read,
    archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
    file_patterns: &[PathBuf],
    wildcards: bool,
    strip_components: u32,
) -> UResult<()> {
    let reader = open_archive_reader(input, compression)?;
    let mut archive = Archive::new(reader);
    let mut out = BufWriter::new(io::stdout().lock());

    if verbose {
        writeln!(out, "Extracting archive: {}", archive_path.display()).map_err(TarError::Io)?;
    }

    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let mut entry = entry_result.map_err(TarError::CannotReadEntry)?;

        let path = entry
            .path()
            .map_err(TarError::CannotReadEntryPath)?
            .to_path_buf();

        if !entry_matches(&path, file_patterns, wildcards) {
            continue;
        }

        if strip_components > 0 {
            let effective_path = match strip_leading_components(&path, strip_components) {
                Some(p) => p,
                None => continue,
            };

            // Reject paths that escape the destination after stripping.
            if effective_path.is_absolute()
                || effective_path
                    .components()
                    .any(|c| c == Component::ParentDir)
            {
                continue;
            }

            if verbose {
                writeln!(out, "{}", effective_path.display()).map_err(TarError::Io)?;
            }

            if let Some(parent) = effective_path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(TarError::Io)?;
                }
            }
            entry
                .unpack(&effective_path)
                .map_err(|e| TarError::CannotExtract {
                    path: effective_path,
                    source: e,
                })?;
        } else {
            if verbose {
                writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
            }

            entry.unpack_in(".").map_err(|e| TarError::CannotExtract {
                path: path.clone(),
                source: e,
            })?;
        }
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
