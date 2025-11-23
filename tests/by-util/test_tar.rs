// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

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
        .code_is(1);
}

// List operation tests

#[test]
fn test_list_archive() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create an archive with multiple files
    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt"])
        .succeeds();

    // Remove originals
    at.remove("file1.txt");
    at.remove("file2.txt");

    // List
    let res = new_ucmd!()
        .arg("-tf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    res.stdout_is("file1.txt\nfile2.txt\n");
}

#[test]
fn test_list_archive_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    let file_names = vec!["file1.txt".to_string(), "file2.txt".to_string()];

    // Create an archive with multiple files
    at.write(&file_names[0], "content1");
    at.write(&file_names[1], "content2");
    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt"])
        .succeeds();

    // Remove originals
    at.remove(&file_names[0]);
    at.remove(&file_names[1]);

    // List
    let res = new_ucmd!()
        .arg("-tvf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    let mut list_files = vec![];

    for line in res.stdout_str().lines() {
        if !line.is_empty() {
            // rev, trim till whilespace, collect, split, rev(again), collect
            // to flip since file name is variable grab the last string in the
            // stdout line
            let file_name = line
                .to_string()
                .chars()
                .rev()
                .take_while(|x| !x.is_whitespace())
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>();
            list_files.push(file_name);
        }
    }

    assert_eq!(file_names, list_files);
}
