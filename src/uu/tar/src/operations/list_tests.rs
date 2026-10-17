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

#[cfg(unix)]
#[test]
fn test_write_member_name_quoting() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let c_cases: &[(&[u8], &[u8])] = &[
        (b"plain name", b"plain name"),
        (b"bad-\xff\xc3\xa9\xfe", b"bad-\\377\\303\\251\\376"),
        ("é".as_bytes(), b"\\303\\251"),
    ];
    let utf8_cases: &[(&[u8], &[u8])] = &[
        (b"bad-\xff", b"bad-\\377"),
        (b"bad-\xc3", b"bad-\\303"),
        (b"a\xffz", b"a\\377z"),
        (b"\xff\xc3\xa9\xfe", "\\377é\\376".as_bytes()),
        (b"x\xe2\x82", b"x\\342\\202"),
        ("normal-é".as_bytes(), "normal-é".as_bytes()),
        (b"bad-\\377", b"bad-\\\\377"),
        (b"bad-\n\t\r\x1b", b"bad-\\n\\t\\r\\033"),
        (b"\x07\x08\x0b\x0c\x7f", b"\\a\\b\\v\\f\\177"),
        (b"bad-\\n\\t", b"bad-\\\\n\\\\t"),
        (b"bad-\xe2\x80\xa8", b"bad-\\342\\200\\250"),
        (b"bad-\xe2\x80\xa9", b"bad-\\342\\200\\251"),
    ];
    for (c_locale, cases) in [(true, c_cases), (false, utf8_cases)] {
        for &(input, expected) in cases {
            let mut output = Vec::new();
            write_member_name(&mut output, Path::new(OsStr::from_bytes(input)), c_locale).unwrap();
            assert_eq!(output, expected, "{input:?} C={c_locale}");
        }
    }
}

#[test]
fn test_member_name_locale_precedence() {
    use std::ffi::OsStr;

    for (all, ctype, lang, expected) in [
        (None, None, None, true),
        (Some("POSIX"), None, None, true),
        (Some("C"), Some("C.UTF-8"), Some("C"), true),
        (Some(""), Some("C.UTF-8"), Some("C"), false),
        (Some(""), Some(""), Some("C"), true),
        (Some(""), Some(""), Some("C.UTF-8"), false),
    ] {
        assert_eq!(
            member_name_c_locale_for(
                all.map(OsStr::new),
                ctype.map(OsStr::new),
                lang.map(OsStr::new),
            ),
            expected
        );
    }
}
