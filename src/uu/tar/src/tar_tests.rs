// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use super::*;
use std::ffi::OsString;
use std::fs;
use tempfile::tempdir;

// --- is_posix_keystring ---

#[test]
fn test_keystring_create() {
    assert!(is_posix_keystring("c"));
    assert!(is_posix_keystring("cf"));
    assert!(is_posix_keystring("cvf"));
    assert!(is_posix_keystring("cv"));
}

#[test]
fn test_keystring_extract() {
    assert!(is_posix_keystring("x"));
    assert!(is_posix_keystring("xf"));
    assert!(is_posix_keystring("xvf"));
}

#[test]
fn test_keystring_rejects_dash_prefix() {
    assert!(!is_posix_keystring("-c"));
    assert!(!is_posix_keystring("-cf"));
    assert!(!is_posix_keystring("-xvf"));
}

#[test]
fn test_keystring_rejects_no_function_letter() {
    // modifier-only strings are not valid keystrings
    assert!(!is_posix_keystring("f"));
    assert!(!is_posix_keystring("vf"));
    assert!(!is_posix_keystring("v"));
}

#[test]
fn test_keystring_rejects_invalid_chars() {
    assert!(!is_posix_keystring("cz")); // 'z' is not a key char
    assert!(!is_posix_keystring("c1")); // digits not allowed
    assert!(!is_posix_keystring("archive.tar")); // typical filename
}

#[test]
fn test_keystring_rejects_empty() {
    assert!(!is_posix_keystring(""));
}

// --- expand_posix_keystring ---

fn osvec(v: &[&str]) -> Vec<std::ffi::OsString> {
    v.iter().map(std::ffi::OsString::from).collect()
}

#[test]
fn test_expand_cf() {
    let input = osvec(&["tar", "cf", "archive.tar", "file.txt"]);
    let expected = osvec(&["tar", "-c", "-f", "archive.tar", "file.txt"]);
    assert_eq!(expand_posix_keystring(input), expected);
}

#[test]
fn test_expand_cvf() {
    let input = osvec(&["tar", "cvf", "archive.tar", "file.txt"]);
    let expected = osvec(&["tar", "-c", "-v", "-f", "archive.tar", "file.txt"]);
    assert_eq!(expand_posix_keystring(input), expected);
}

#[test]
fn test_expand_xf() {
    let input = osvec(&["tar", "xf", "archive.tar"]);
    let expected = osvec(&["tar", "-x", "-f", "archive.tar"]);
    assert_eq!(expand_posix_keystring(input), expected);
}

#[test]
fn test_expand_xvf() {
    let input = osvec(&["tar", "xvf", "archive.tar"]);
    let expected = osvec(&["tar", "-x", "-v", "-f", "archive.tar"]);
    assert_eq!(expand_posix_keystring(input), expected);
}

#[test]
fn test_expand_preserves_dash_prefix_args() {
    // When args already use '-' prefixes, no expansion should occur
    let input = osvec(&["tar", "-cvf", "archive.tar", "file.txt"]);
    assert_eq!(expand_posix_keystring(input.clone()), input);
}

#[test]
fn test_expand_f_before_files() {
    // 'f' consumes only the archive name; remaining args are files
    let input = osvec(&["tar", "cf", "archive.tar", "a.txt", "b.txt"]);
    let expected = osvec(&["tar", "-c", "-f", "archive.tar", "a.txt", "b.txt"]);
    assert_eq!(expand_posix_keystring(input), expected);
}

#[test]
fn test_expand_function_letter_only() {
    // No 'f' modifier: no archive consumed from file operands
    let input = osvec(&["tar", "c", "file.txt"]);
    let expected = osvec(&["tar", "-c", "file.txt"]);
    assert_eq!(expand_posix_keystring(input), expected);
}

#[test]
fn test_expand_cbf() {
    let input = osvec(&["tar", "cbf", "20", "archive.tar", "file.txt"]);
    let expected = osvec(&["tar", "-c", "-b", "20", "-f", "archive.tar", "file.txt"]);
    assert_eq!(expand_posix_keystring(input), expected);
}

#[test]
fn test_uumain_dispatches_zstd_create_list_extract() {
    let tempdir = tempdir().unwrap();
    let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
    fs::write("file.txt", "hello").unwrap();

    let create_args = vec![
        OsString::from("test-bin"),
        OsString::from("tar"),
        OsString::from("--zstd"),
        OsString::from("-cf"),
        OsString::from("archive.tar.zst"),
        OsString::from("file.txt"),
    ];
    assert_eq!(uumain(create_args.into_iter()), 0);

    let list_args = vec![
        OsString::from("test-bin"),
        OsString::from("tar"),
        OsString::from("--zstd"),
        OsString::from("-tf"),
        OsString::from("archive.tar.zst"),
    ];
    assert_eq!(uumain(list_args.into_iter()), 0);

    fs::remove_file("file.txt").unwrap();
    let extract_args = vec![
        OsString::from("test-bin"),
        OsString::from("tar"),
        OsString::from("--zstd"),
        OsString::from("-xf"),
        OsString::from("archive.tar.zst"),
    ];
    let result = uumain(extract_args.into_iter());

    assert_eq!(result, 0);
    assert_eq!(
        fs::read_to_string(tempdir.path().join("file.txt")).unwrap(),
        "hello"
    );
}
