// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

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
fn test_create_archive_gzip_flush_failed() {
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
        CompressionMode::Gzip,
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
