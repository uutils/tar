// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::io;
use std::path::PathBuf;
use uu_tar::errors::TarError;

#[test]
fn test_tar_error_display() {
    let err = TarError::FileNotFound {
        path: PathBuf::from("test.txt"),
    };
    assert_eq!(
        err.to_string(),
        "test.txt: Cannot open: No such file or directory"
    );

    let err = TarError::PermissionDenied {
        path: PathBuf::from("/root/file"),
    };
    assert_eq!(
        err.to_string(),
        "/root/file: Cannot open: Permission denied"
    );
}

#[test]
fn test_tar_error_code() {
    use uucore::error::UError;

    assert_eq!(
        TarError::FileNotFound {
            path: PathBuf::from("test")
        }
        .code(),
        2
    );
    assert_eq!(
        TarError::PermissionDenied {
            path: PathBuf::from("test")
        }
        .code(),
        2
    );
    assert_eq!(
        TarError::Io(io::Error::new(io::ErrorKind::Other, "test")).code(),
        2
    );
}

#[test]
fn test_io_error_conversion_not_found() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let tar_err = TarError::from(io_err);

    match tar_err {
        TarError::Io(e) => assert_eq!(e.kind(), io::ErrorKind::NotFound),
        _ => panic!("Expected Io variant"),
    }
}

#[test]
fn test_io_error_conversion_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let tar_err = TarError::from(io_err);

    match tar_err {
        TarError::Io(e) => assert_eq!(e.kind(), io::ErrorKind::PermissionDenied),
        _ => panic!("Expected Io variant"),
    }
}

#[test]
fn test_io_error_conversion_other() {
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
    let tar_err = TarError::from(io_err);

    match tar_err {
        TarError::Io(e) => assert_eq!(e.kind(), io::ErrorKind::BrokenPipe),
        _ => panic!("Expected Io variant"),
    }
}

#[test]
fn test_error_source() {
    let io_err = io::Error::other("some error");
    let tar_err = TarError::Io(io_err);

    // Io should have a source
    assert!(std::error::Error::source(&tar_err).is_some());

    // Other variants should not have a source
    let tar_err = TarError::FileNotFound {
        path: PathBuf::from("test"),
    };
    assert!(std::error::Error::source(&tar_err).is_none());
}

#[test]
fn test_from_io_error_not_found() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
    let tar_err = TarError::from_io_error(io_err, std::path::Path::new("myfile.txt"));

    assert!(matches!(tar_err, TarError::FileNotFound { .. }));
    assert_eq!(
        tar_err.to_string(),
        "myfile.txt: Cannot open: No such file or directory"
    );
}

#[test]
fn test_from_io_error_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let tar_err = TarError::from_io_error(io_err, std::path::Path::new("/root/secret"));

    assert!(matches!(tar_err, TarError::PermissionDenied { .. }));
    assert_eq!(
        tar_err.to_string(),
        "/root/secret: Cannot open: Permission denied"
    );
}

#[test]
fn test_from_io_error_other() {
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "broken");
    let tar_err = TarError::from_io_error(io_err, std::path::Path::new("file.txt"));

    assert!(matches!(tar_err, TarError::Io(_)));
}

#[test]
fn test_tar_error_is_debug() {
    let err = TarError::FileNotFound {
        path: PathBuf::from("test"),
    };
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("FileNotFound"));
    assert!(debug_str.contains("test"));
}
