// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::io;
use std::path::PathBuf;
use thiserror::Error;
use uucore::error::UError;

/// Error types for tar operations
#[derive(Debug, Error)]
pub enum TarError {
    /// I/O error occurred while reading/writing
    #[error("{0}")]
    Io(#[from] io::Error),

    /// Cannot read entries from archive
    #[error("tar: Cannot read archive entries: {0}")]
    CannotReadEntries(io::Error),

    /// Cannot read an individual archive entry
    #[error("tar: Cannot read archive entry: {0}")]
    CannotReadEntry(io::Error),

    /// Cannot read the path of an archive entry
    #[error("tar: Cannot read entry path: {0}")]
    CannotReadEntryPath(io::Error),

    /// File or directory not found
    #[error("{path}: Cannot open: No such file or directory")]
    FileNotFound { path: PathBuf },

    /// Permission denied when accessing file
    #[error("{path}: Cannot open: Permission denied")]
    PermissionDenied { path: PathBuf },

    /// Cannot create archive file
    #[error("tar: Cannot create archive '{path}': {source}")]
    CannotCreateArchive { path: PathBuf, source: io::Error },

    /// Cannot open archive file
    #[error("tar: Cannot open archive '{path}': {source}")]
    CannotOpenArchive { path: PathBuf, source: io::Error },

    /// Cannot add a directory to the archive
    #[error("tar: Cannot add directory '{path}': {source}")]
    CannotAddDirectory { path: PathBuf, source: io::Error },

    /// Cannot add a file to the archive
    #[error("tar: Cannot add file '{path}': {source}")]
    CannotAddFile { path: PathBuf, source: io::Error },

    /// Cannot extract an archive entry
    #[error("tar: Cannot extract '{path}': {source}")]
    CannotExtract { path: PathBuf, source: io::Error },

    /// Cannot finalize the archive
    #[error("tar: Cannot finalize archive: {0}")]
    CannotFinalizeArchive(io::Error),
}

impl TarError {
    /// Create a TarError from an io::Error with file path context
    pub fn from_io_error(err: io::Error, path: &std::path::Path) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => TarError::FileNotFound {
                path: path.to_path_buf(),
            },
            io::ErrorKind::PermissionDenied => TarError::PermissionDenied {
                path: path.to_path_buf(),
            },
            _ => TarError::Io(err),
        }
    }
}

impl UError for TarError {
    /// Returns the exit code for this error type
    fn code(&self) -> i32 {
        2 // TarError variants exit with code 2; argument/usage errors use code 64 (see tar.rs)
    }
}
