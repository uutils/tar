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

/// Write a minimal POSIX tar archive as raw bytes, bypassing the `tar` crate's path
/// validation. Real-world malicious archives can contain `..` or absolute path components
/// that the `tar` builder refuses to create — this helper lets us test those cases.
fn make_raw_traversal_tar(archive_path: &std::path::Path, entry_name: &[u8], content: &[u8]) {
    use std::io::Write;

    let mut header = [0u8; 512];
    let n = entry_name.len().min(100);
    header[..n].copy_from_slice(&entry_name[..n]);
    header[100..108].copy_from_slice(b"0000644\0");
    header[108..116].copy_from_slice(b"0000000\0");
    header[116..124].copy_from_slice(b"0000000\0");
    let size_str = format!("{:011o}\0", content.len());
    header[124..136].copy_from_slice(size_str.as_bytes());
    header[136..148].copy_from_slice(b"00000000000\0");
    header[148..156].fill(b' '); // spaces for checksum calculation
    header[156] = b'0'; // regular file
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum: u32 = header.iter().map(|&b| b as u32).sum();
    let cs = format!("{:06o}\0 ", checksum);
    header[148..156].copy_from_slice(cs.as_bytes());

    let pad = (512 - (content.len() % 512)) % 512;
    let mut file = fs::File::create(archive_path).unwrap();
    file.write_all(&header).unwrap();
    file.write_all(content).unwrap();
    file.write_all(&vec![0u8; pad]).unwrap();
    file.write_all(&[0u8; 1024]).unwrap(); // end-of-archive
}

#[test]
fn test_extract_strip_components_verbose() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    make_zstd_tar(&archive_path, &[("prefix/file.txt", "content")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        true,
        CompressionMode::Zstd,
        &[],
        false,
        1,
    )
    .unwrap();

    assert!(tempdir.path().join("file.txt").exists());
}

#[test]
fn test_extract_strip_creates_parent_dirs() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    // strip 1 → "subdir/file.txt"; parent "subdir" is non-empty → create_dir_all
    make_zstd_tar(&archive_path, &[("prefix/subdir/file.txt", "content")]);

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[],
        false,
        1,
    )
    .unwrap();

    assert!(tempdir.path().join("subdir/file.txt").exists());
}

#[test]
fn test_extract_strip_rejects_path_traversal() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar");
    // After stripping "prefix", the remaining path "../../../escape.txt" contains ".." → skip.
    // We write raw bytes to bypass the `tar` crate's builder validation (which also rejects "..").
    make_raw_traversal_tar(&archive_path, b"prefix/../../../escape.txt", b"evil");

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    extract_archive(
        input,
        &archive_path,
        false,
        CompressionMode::None,
        &[],
        false,
        1,
    )
    .unwrap();

    assert!(!tempdir.path().join("escape.txt").exists());
}
