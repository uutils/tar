// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use super::*;
use crate::CompressionMode;
use std::fs;
use tar::Builder;
use tempfile::tempdir;

fn write_zstd_tar(archive_path: &Path) {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size("hello".len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, "listed.txt", std::io::Cursor::new("hello"))
            .unwrap();
        builder.finish().unwrap();
    }
    let compressed = zstd::stream::encode_all(std::io::Cursor::new(tar_bytes), 0).unwrap();
    fs::write(archive_path, compressed).unwrap();
}

#[test]
fn test_list_archive_with_zstd_non_verbose() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    write_zstd_tar(&archive_path);

    let input = fs::File::open(&archive_path).unwrap();
    list_archive(input, &archive_path, false, CompressionMode::Zstd).unwrap();
}

#[test]
fn test_list_archive_with_zstd_verbose() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");
    write_zstd_tar(&archive_path);

    let input = fs::File::open(&archive_path).unwrap();
    list_archive(input, &archive_path, true, CompressionMode::Zstd).unwrap();
}
