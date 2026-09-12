// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use super::*;
use crate::{BackupControl, CompressionMode};
use std::fs;
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
                .append_data(&mut header, name, std::io::Cursor::new(*content))
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
        BackupControl::None,
        "~",
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(tempdir.path().join("extracted.txt")).unwrap(),
        "hello"
    );
}

// --- backup: None (default, file overwritten) ---

#[test]
fn test_backup_none_overwrites_existing() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "new")]);
    fs::write(tempdir.path().join("file.txt"), "old").unwrap();

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::None,
        "~",
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt")).unwrap(),
        "new"
    );
    assert!(!tempdir.path().join("file.txt~").exists());
}

// --- backup: Simple ---

#[test]
fn test_backup_simple_renames_existing() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "new")]);
    fs::write(tempdir.path().join("file.txt"), "old").unwrap();

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::Simple,
        "~",
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt")).unwrap(),
        "new"
    );
    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt~")).unwrap(),
        "old"
    );
}

#[test]
fn test_backup_simple_custom_suffix() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "new")]);
    fs::write(tempdir.path().join("file.txt"), "old").unwrap();

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::Simple,
        ".bak",
    )
    .unwrap();

    assert!(tempdir.path().join("file.txt.bak").exists());
}

#[test]
fn test_backup_simple_no_op_when_no_existing_file() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "new")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::Simple,
        "~",
    )
    .unwrap();

    assert!(tempdir.path().join("file.txt").exists());
    assert!(!tempdir.path().join("file.txt~").exists());
}

// --- backup: Numbered ---

#[test]
fn test_backup_numbered_first_backup() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "new")]);
    fs::write(tempdir.path().join("file.txt"), "old").unwrap();

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::Numbered,
        "~",
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt")).unwrap(),
        "new"
    );
    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt.~1~")).unwrap(),
        "old"
    );
}

#[test]
fn test_backup_numbered_increments() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    fs::write(tempdir.path().join("file.txt"), "v1").unwrap();
    fs::write(tempdir.path().join("file.txt.~1~"), "v0").unwrap();
    make_zstd_tar(&archive_path, &[("file.txt", "v2")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::Numbered,
        "~",
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt.~2~")).unwrap(),
        "v1"
    );
    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt.~1~")).unwrap(),
        "v0"
    );
}

// --- backup: Existing ---

#[test]
fn test_backup_existing_uses_simple_when_no_numbered_backup() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "new")]);
    fs::write(tempdir.path().join("file.txt"), "old").unwrap();

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::Existing,
        "~",
    )
    .unwrap();

    // no .~1~ existed → simple backup
    assert!(tempdir.path().join("file.txt~").exists());
    assert!(!tempdir.path().join("file.txt.~1~").exists());
}

#[test]
fn test_backup_existing_uses_numbered_when_numbered_backup_present() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("file.txt", "new")]);
    fs::write(tempdir.path().join("file.txt"), "old").unwrap();
    fs::write(tempdir.path().join("file.txt.~1~"), "older").unwrap();

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        BackupControl::Existing,
        "~",
    )
    .unwrap();

    // .~1~ existed → numbered mode → backup goes to .~2~
    assert!(tempdir.path().join("file.txt.~2~").exists());
    assert!(!tempdir.path().join("file.txt~").exists());
}
