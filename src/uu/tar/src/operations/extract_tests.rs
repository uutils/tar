// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use super::*;
use crate::CompressionMode;
use std::fs;
use std::path::PathBuf;
use tar::Builder;
use tempfile::tempdir;

fn make_zstd_tar(archive_path: &std::path::Path, entries: &[(&str, &str)]) {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        for (name, content) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(content.len() as u64);
            header.set_cksum();
            builder
                .append_data(&mut header, name, std::io::Cursor::new(content))
                .unwrap();
        }
        builder.finish().unwrap();
    }
    let compressed = zstd::stream::encode_all(std::io::Cursor::new(tar_bytes), 0).unwrap();
    fs::write(archive_path, compressed).unwrap();
}

#[test]
fn test_extract_archive_with_zstd() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("extracted.txt", "hello")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        true,
        CompressionMode::Zstd,
        &[],
        false,
        0,
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(tempdir.path().join("extracted.txt")).unwrap(),
        "hello"
    );
}

#[test]
fn test_extract_with_file_pattern_filter() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("keep.txt", "keep"), ("skip.txt", "skip")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[PathBuf::from("keep.txt")],
        false,
        0,
    )
    .unwrap();

    assert!(tempdir.path().join("keep.txt").exists());
    assert!(!tempdir.path().join("skip.txt").exists());
}

#[test]
fn test_extract_with_wildcards() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "txt"), ("file.rs", "rs")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[PathBuf::from("*.txt")],
        true,
        0,
    )
    .unwrap();

    assert!(tempdir.path().join("file.txt").exists());
    assert!(!tempdir.path().join("file.rs").exists());
}

#[test]
fn test_extract_with_strip_components() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("a/b/file.txt", "content")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[],
        false,
        2,
    )
    .unwrap();

    assert!(tempdir.path().join("file.txt").exists());
    assert!(!tempdir.path().join("a").exists());
}

#[test]
fn test_extract_strip_components_skips_shallow_entries() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    // "root.txt" has only 1 component; stripping 2 should skip it entirely
    make_zstd_tar(
        &archive_path,
        &[("root.txt", "shallow"), ("a/b/deep.txt", "deep")],
    );

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[],
        false,
        2,
    )
    .unwrap();

    assert!(!tempdir.path().join("root.txt").exists());
    assert!(tempdir.path().join("deep.txt").exists());
}
