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

fn write_zstd_tar(archive_path: &Path, entries: &[(&str, &str)]) {
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
fn test_list_archive_with_zstd_non_verbose() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    write_zstd_tar(&archive_path, &[("listed.txt", "hello")]);

    let input = fs::File::open(&archive_path).unwrap();
    list_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[],
        false,
        0,
    )
    .unwrap();
}

#[test]
fn test_list_archive_with_zstd_verbose() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    write_zstd_tar(&archive_path, &[("listed.txt", "hello")]);

    let input = fs::File::open(&archive_path).unwrap();
    list_archive(
        input,
        &archive_path,
        true,
        CompressionMode::Zstd,
        &[],
        false,
        0,
    )
    .unwrap();
}

#[test]
fn test_list_with_file_pattern_filter() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    write_zstd_tar(&archive_path, &[("a.txt", "a"), ("b.txt", "b")]);

    let input = fs::File::open(&archive_path).unwrap();
    // Should not panic; filtering is tested at the integration level
    list_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[PathBuf::from("a.txt")],
        false,
        0,
    )
    .unwrap();
}

#[test]
fn test_list_with_wildcards() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    write_zstd_tar(&archive_path, &[("foo.txt", "f"), ("bar.rs", "b")]);

    let input = fs::File::open(&archive_path).unwrap();
    list_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[PathBuf::from("*.txt")],
        true,
        0,
    )
    .unwrap();
}

#[test]
fn test_list_with_strip_components() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    write_zstd_tar(&archive_path, &[("a/b/file.txt", "x")]);

    let input = fs::File::open(&archive_path).unwrap();
    list_archive(
        input,
        &archive_path,
        false,
        CompressionMode::Zstd,
        &[],
        false,
        2,
    )
    .unwrap();
}
