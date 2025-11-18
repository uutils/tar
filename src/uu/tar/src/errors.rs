// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt;
use std::io;
use uucore::error::UError;

/// Error types for tar operations
#[derive(Debug)]
pub enum TarError {
    /// I/O error occurred
    IoError(io::Error),
    /// Invalid archive format or corrupted archive
    InvalidArchive(String),
    /// File or directory not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// General tar operation error
    TarOperationError(String),
}

/// Implements display formatting for TarError.
impl fmt::Display for TarError {
    /// Formats the error for display to users
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TarError::IoError(err) => write!(f, "I/O error: {err}"),
            TarError::InvalidArchive(msg) => write!(f, "Invalid archive: {msg}"),
            TarError::FileNotFound(path) => write!(f, "File not found: {path}"),
            TarError::PermissionDenied(path) => write!(f, "Permission denied: {path}"),
            TarError::TarOperationError(msg) => write!(f, "tar: {msg}"),
        }
    }
}

impl std::error::Error for TarError {
    /// Returns the underlying error cause, if any
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TarError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl UError for TarError {
    /// Returns the exit code for this error type
    fn code(&self) -> i32 {
        match self {
            TarError::IoError(_) => 1,
            TarError::InvalidArchive(_) => 2,
            TarError::FileNotFound(_) => 1,
            TarError::PermissionDenied(_) => 1,
            TarError::TarOperationError(_) => 1,
        }
    }
}

impl From<io::Error> for TarError {
    /// Converts io::Error into the appropriate TarError variant
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => TarError::FileNotFound(err.to_string()),
            io::ErrorKind::PermissionDenied => TarError::PermissionDenied(err.to_string()),
            _ => TarError::IoError(err),
        }
    }
}
