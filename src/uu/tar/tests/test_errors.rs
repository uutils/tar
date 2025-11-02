// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::io;
use uu_tar::errors::TarError;

#[test]
fn test_tar_error_display() {
    let err = TarError::FileNotFound("test.txt".to_string());
    assert_eq!(err.to_string(), "File not found: test.txt");

    let err = TarError::InvalidArchive("corrupted header".to_string());
    assert_eq!(err.to_string(), "Invalid archive: corrupted header");

    let err = TarError::PermissionDenied("/root/file".to_string());
    assert_eq!(err.to_string(), "Permission denied: /root/file");

    let err = TarError::TarOperationError("failed to write".to_string());
    assert_eq!(err.to_string(), "tar: failed to write");
}

#[test]
fn test_tar_error_code() {
    use uucore::error::UError;

    assert_eq!(TarError::FileNotFound("test".to_string()).code(), 1);
    assert_eq!(TarError::InvalidArchive("test".to_string()).code(), 2);
    assert_eq!(TarError::PermissionDenied("test".to_string()).code(), 1);
    assert_eq!(TarError::TarOperationError("test".to_string()).code(), 1);
}

#[test]
fn test_io_error_conversion_not_found() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let tar_err = TarError::from(io_err);

    match tar_err {
        TarError::FileNotFound(msg) => assert!(msg.contains("file not found")),
        _ => panic!("Expected FileNotFound variant"),
    }
}

#[test]
fn test_io_error_conversion_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let tar_err = TarError::from(io_err);

    match tar_err {
        TarError::PermissionDenied(msg) => assert!(msg.contains("access denied")),
        _ => panic!("Expected PermissionDenied variant"),
    }
}

#[test]
fn test_io_error_conversion_other() {
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
    let tar_err = TarError::from(io_err);

    match tar_err {
        TarError::IoError(e) => assert_eq!(e.kind(), io::ErrorKind::BrokenPipe),
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_error_source() {
    let io_err = io::Error::other("some error");
    let tar_err = TarError::IoError(io_err);

    // IoError should have a source
    assert!(std::error::Error::source(&tar_err).is_some());

    // Other variants should not have a source
    let tar_err = TarError::FileNotFound("test".to_string());
    assert!(std::error::Error::source(&tar_err).is_none());
}

#[test]
fn test_tar_error_is_debug() {
    let err = TarError::TarOperationError("test".to_string());
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("TarOperationError"));
    assert!(debug_str.contains("test"));
}
