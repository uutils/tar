// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use super::*;
use crate::CompressionMode;
use std::fs;
use tar::Builder;
use tempfile::tempdir;

#[test]
fn test_extract_archive_with_zstd() {
    let tempdir = tempdir().unwrap();
    let archive_path = tempdir.path().join("archive.tar.zst");

    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size("hello".len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, "extracted.txt", std::io::Cursor::new("hello"))
            .unwrap();
        builder.finish().unwrap();
    }
    let compressed = zstd::stream::encode_all(std::io::Cursor::new(tar_bytes), 0).unwrap();
    fs::write(&archive_path, compressed).unwrap();

    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    let input = fs::File::open(&archive_path).unwrap();
    let result = extract_archive(input, &archive_path, true, CompressionMode::Zstd);

    result.unwrap();
    assert_eq!(
        fs::read_to_string(tempdir.path().join("extracted.txt")).unwrap(),
        "hello"
    );
}
