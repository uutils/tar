// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::TarError;
use crate::{BackupControl, CompressionMode};
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use uucore::error::UResult;

fn simple_backup_path(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

fn numbered_backup_path(path: &Path) -> PathBuf {
    for n in 1u64.. {
        let mut s = path.as_os_str().to_owned();
        s.push(format!(".~{n}~"));
        let candidate = PathBuf::from(&s);
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}

fn backup_file(path: &Path, control: BackupControl, suffix: &str) -> Result<(), TarError> {
    if matches!(control, BackupControl::None) || !path.is_file() {
        return Ok(());
    }
    let backup = match control {
        BackupControl::None => unreachable!(),
        BackupControl::Simple => simple_backup_path(path, suffix),
        BackupControl::Numbered => numbered_backup_path(path),
        BackupControl::Existing => {
            // use numbered if a .~1~ backup already exists, otherwise simple
            let first_numbered = simple_backup_path(path, ".~1~");
            if first_numbered.exists() {
                numbered_backup_path(path)
            } else {
                simple_backup_path(path, suffix)
            }
        }
    };
    std::fs::rename(path, &backup).map_err(|e| TarError::CannotBackup {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Extract files from a tar archive.
///
/// # Arguments
///
/// * `input` - Readable source of the archive data
/// * `archive_path` - Path used for error messages and verbose header
/// * `verbose` - Print each extracted path to stdout
/// * `compression` - Compression mode to use when reading
/// * `backup_control` - Whether and how to back up existing files before overwriting
/// * `backup_suffix` - Suffix used for simple backups (default: `~`)
///
/// # Errors
///
/// Returns an error if the archive cannot be read or entries cannot be extracted.
pub fn extract_archive(
    input: impl Read,
    archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
    backup_control: BackupControl,
    backup_suffix: &str,
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

        if verbose {
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }

        backup_file(&path, backup_control, backup_suffix)?;

        entry.unpack_in(".").map_err(|e| TarError::CannotExtract {
            path: path.clone(),
            source: e,
        })?;
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
