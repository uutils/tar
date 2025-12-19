// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::path::{self, PathBuf};

use uutests::{at_and_ucmd, new_ucmd};

// Basic CLI Tests

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_help() {
    new_ucmd!()
        .arg("--help")
        .succeeds()
        .code_is(0)
        .stdout_contains("an archiving utility");
}

#[test]
fn test_version() {
    new_ucmd!()
        .arg("--version")
        .succeeds()
        .code_is(0)
        .stdout_contains("tar");
}

#[test]
fn test_create_dir_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    let separator = path::MAIN_SEPARATOR;
    let dir1_path = "dir1";
    let dir2_path = format!("{dir1_path}{separator}dir2");
    let file1_path = format!("{dir1_path}{separator}file1.txt");
    let file2_path = format!("{dir2_path}{separator}file2.txt");

    at.mkdir(dir1_path);
    at.mkdir(&dir2_path);
    at.write(&file1_path, "test content 1");
    at.write(&file2_path, "test content 2");

    ucmd.args(&["-cvf", "archive.tar", dir1_path])
        .succeeds()
        .stdout_contains(dir1_path)
        .stdout_contains(dir2_path)
        .stdout_contains(file1_path)
        .stdout_contains(file2_path);
}

// Create operation tests

#[test]
fn test_create_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "test content");

    ucmd.args(&["-cf", "archive.tar", "file1.txt"]).succeeds();

    assert!(at.file_exists("archive.tar"));
}

#[test]
fn test_create_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");

    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt"])
        .succeeds();

    assert!(at.file_exists("archive.tar"));
}

#[test]
fn test_create_absolute_path() {
    let (at, mut ucmd) = at_and_ucmd!();

    let mut file_abs_path = PathBuf::from(at.root_dir_resolved());
    file_abs_path.push("file1.txt");

    at.write(&file_abs_path.display().to_string(), "content1");
    ucmd.args(&["-cf", "archive.tar", &file_abs_path.display().to_string()])
        .succeeds()
        .stdout_contains("Removing leading");

    assert!(at.file_exists("archive.tar"));
}

// Extract operation tests

#[test]
fn test_extract_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create an archive first
    at.write("original.txt", "test content");
    ucmd.args(&["-cf", "archive.tar", "original.txt"])
        .succeeds();

    // Remove original and extract
    at.remove("original.txt");

    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("original.txt"));
    assert_eq!(at.read("original.txt"), "test content");
}

#[test]
fn test_extract_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create an archive with multiple files
    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt"])
        .succeeds();

    // Remove originals
    at.remove("file1.txt");
    at.remove("file2.txt");

    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("file1.txt"));
    assert!(at.file_exists("file2.txt"));
    assert_eq!(at.read("file1.txt"), "content1");
    assert_eq!(at.read("file2.txt"), "content2");
}

#[test]
fn test_extract_nonexistent_archive() {
    new_ucmd!()
        .args(&["-xf", "nonexistent.tar"])
        .fails()
        .code_is(2);
}

#[test]
fn test_extract_created_from_absolute_path() {
    let (at, mut ucmd) = at_and_ucmd!();

    let mut file_abs_path = PathBuf::from(at.root_dir_resolved());
    file_abs_path.push("file1.txt");

    at.write(&file_abs_path.display().to_string(), "content1");
    ucmd.args(&["-cf", "archive.tar", &file_abs_path.display().to_string()])
        .succeeds();

    new_ucmd!()
        .args(&["-xf", "archive.tar"])
        .current_dir(at.as_string())
        .succeeds();

    let expected_path = file_abs_path
        .components()
        .filter(|c| !matches!(c, path::Component::RootDir | path::Component::Prefix(_)))
        .map(|c| c.as_os_str().display().to_string())
        .collect::<Vec<_>>()
        .join(&path::MAIN_SEPARATOR.to_string());

    assert!(at.file_exists(expected_path));
}
