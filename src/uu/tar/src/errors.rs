// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::io;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;
use uucore::error::{set_exit_code, UError, UResult};

/// Error types for tar operations
#[derive(Debug, Error)]
pub enum TarError {
    /// I/O error occurred while reading/writing
    #[error("{0}")]
    Io(#[from] io::Error),

    /// Cannot read entries from archive
    #[error("Cannot read archive entries: {0}")]
    CannotReadEntries(io::Error),

    /// Cannot read an individual archive entry
    #[error("Cannot read archive entry: {0}")]
    CannotReadEntry(io::Error),

    /// Cannot read the path of an archive entry
    #[error("Cannot read entry path: {0}")]
    CannotReadEntryPath(io::Error),

    /// Invalid archive format or unsupported compression stream
    #[error("{0}")]
    InvalidArchive(String),

    /// File or directory not found
    #[error("{path}: Cannot open: No such file or directory")]
    FileNotFound { path: PathBuf },

    /// Permission denied when accessing file
    #[error("{path}: Cannot open: Permission denied")]
    PermissionDenied { path: PathBuf },

    /// Cannot create archive file
    #[error("Cannot create archive '{path}': {source}")]
    CannotCreateArchive { path: PathBuf, source: io::Error },

    /// Cannot add a directory to the archive
    #[error("Cannot add directory '{path}': {source}")]
    CannotAddDirectory { path: PathBuf, source: io::Error },

    /// Cannot add a file to the archive
    #[error("Cannot add file '{path}': {source}")]
    CannotAddFile { path: PathBuf, source: io::Error },

    /// Cannot extract an archive entry
    #[error("Cannot extract '{path}': {source}")]
    CannotExtract { path: PathBuf, source: io::Error },

    /// General tar operation error
    #[error("{0}")]
    TarOperationError(String),

    /// Cannot finalize the archive
    #[error("Cannot finalize archive: {0}")]
    CannotFinalizeArchive(io::Error),

    /// Refusing to write archive contents to terminal
    #[error("Refusing to write archive contents to terminal (missing -f option?)")]
    RefuseWriteArchiveToTerminal,

    /// A member name with a `..` component, which is not extracted
    #[error("{path}: Member name contains '..'")]
    MemberNameContainsDotDot { path: PathBuf },

    /// Errors that were reported earlier and recovered from
    #[error("Exiting with failure status due to previous errors")]
    PreviousErrors,
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

/// Reports an error that tar recovers from: the operation carries on, and has to
/// end with [`finish_after_previous_errors`].
pub fn report_recoverable(err: &TarError) -> UResult<()> {
    writeln!(io::stderr(), "{}: {err}", uucore::util_name()).map_err(TarError::Io)?;
    Ok(())
}

/// Ends an operation that reported recoverable errors with exit status 2, like
/// GNU tar. The status is set right before returning `Ok`, which is how `tarapp`
/// tells these errors apart from fatal ones.
pub fn finish_after_previous_errors() -> UResult<()> {
    report_recoverable(&TarError::PreviousErrors)?;
    set_exit_code(2);
    Ok(())
}

impl UError for TarError {
    /// Returns the exit code for this error type
    fn code(&self) -> i32 {
        2 // TarError variants exit with code 2; argument/usage errors use code 64 (see tar.rs)
    }
}
