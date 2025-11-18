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
            TarError::IoError(err) => write!(f, "{err}"),
            TarError::InvalidArchive(msg) => write!(f, "{msg}"),
            TarError::FileNotFound(path) => {
                write!(f, "{path}: Cannot open: No such file or directory")
            }
            TarError::PermissionDenied(path) => {
                write!(f, "{path}: Cannot open: Permission denied")
            }
            TarError::TarOperationError(msg) => write!(f, "{msg}"),
        }
    }
}

impl TarError {
    /// Create a TarError from an io::Error with file path context
    pub fn from_io_error(err: io::Error, path: &std::path::Path) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => TarError::FileNotFound(path.display().to_string()),
            io::ErrorKind::PermissionDenied => {
                TarError::PermissionDenied(path.display().to_string())
            }
            _ => TarError::IoError(err),
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
            TarError::IoError(_) => 2,
            TarError::InvalidArchive(_) => 2,
            TarError::FileNotFound(_) => 2,
            TarError::PermissionDenied(_) => 2,
            TarError::TarOperationError(_) => 2,
        }
    }
}

impl From<io::Error> for TarError {
    /// Converts io::Error into the appropriate TarError variant
    fn from(err: io::Error) -> Self {
        // For generic io::Error without context, just wrap it
        TarError::IoError(err)
    }
}
