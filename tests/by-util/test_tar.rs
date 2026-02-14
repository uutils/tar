// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::path::{self, PathBuf};

use uutests::{at_and_ucmd, new_ucmd};

/// Size of a single tar block in bytes (per POSIX specification).
// TODO: This should be exported from the tar crate instead of being redefined here.
// The tar crate has `BLOCK_SIZE` defined but it's marked `pub(crate)`.
const TAR_BLOCK_SIZE: usize = 512;

// Basic CLI tests

#[test]
fn test_invalid_arg() {
    new_ucmd!()
        .arg("--definitely-invalid")
        .fails()
        .code_is(64)
        .stderr_contains("unexpected argument");
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
fn test_conflicting_operations() {
    new_ucmd!()
        .args(&["-c", "-x", "-f", "archive.tar"])
        .fails()
        .code_is(2)
        .stderr_contains("cannot be used with");
}

#[test]
fn test_no_operation_specified() {
    new_ucmd!()
        .args(&["-f", "archive.tar"])
        .fails()
        .code_is(2)
        .stderr_contains("must specify one");
}

// Create operation tests
fn test_create_dir_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    // TODO(jeffbailey): Use PathBuf here instead
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

#[test]
fn test_create_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "test content");

    ucmd.args(&["-cf", "archive.tar", "file1.txt"])
        .succeeds()
        .no_output();

    assert!(at.file_exists("archive.tar"));
    assert!(at.read_bytes("archive.tar").len() > TAR_BLOCK_SIZE); // Basic sanity check
}

#[test]
fn test_create_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    at.write("file3.txt", "content3");

    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds()
        .no_output();

    assert!(at.file_exists("archive.tar"));
    assert!(at.read_bytes("archive.tar").len() > TAR_BLOCK_SIZE); // Basic sanity check
}

#[test]
fn test_create_directory() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.mkdir("dir1");
    at.write("dir1/file1.txt", "content1");
    at.write("dir1/file2.txt", "content2");
    at.mkdir("dir1/subdir");
    at.write("dir1/subdir/file3.txt", "content3");

    ucmd.args(&["-cf", "archive.tar", "dir1"])
        .succeeds()
        .no_output();

    assert!(at.file_exists("archive.tar"));
    assert!(at.read_bytes("archive.tar").len() > TAR_BLOCK_SIZE); // Basic sanity check
}

#[test]
fn test_create_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content");

    ucmd.args(&["-cvf", "archive.tar", "file1.txt"])
        .succeeds()
        .stdout_contains("file1.txt");

    assert!(at.file_exists("archive.tar"));
}

#[test]
fn test_create_empty_archive_fails() {
    new_ucmd!()
        .args(&["-cf", "archive.tar"])
        .fails()
        .code_is(2)
        .stderr_contains("empty archive");
}

#[test]
fn test_create_nonexistent_file_fails() {
    new_ucmd!()
        .args(&["-cf", "archive.tar", "nonexistent.txt"])
        .fails()
        .code_is(2)
        .stderr_contains("nonexistent.txt");
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
        .succeeds()
        .no_output();

    assert!(at.file_exists("original.txt"));
    assert_eq!(at.read("original.txt"), "test content");
}

#[test]
fn test_extract_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create an archive with multiple files
    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    at.write("file3.txt", "content3");

    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds();

    at.remove("file1.txt");
    at.remove("file2.txt");
    at.remove("file3.txt");

    // Extract with verbose
    let result = new_ucmd!()
        .arg("-xvf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    let stdout = result.stdout_str();

    // Verify all files are listed in verbose output
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));
    assert!(stdout.contains("file3.txt"));

    // Verify files were extracted
    assert!(at.file_exists("file1.txt"));
    assert!(at.file_exists("file2.txt"));
    assert!(at.file_exists("file3.txt"));
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
        .code_is(2)
        .stderr_contains("nonexistent.tar");
}

#[test]
fn test_extract_directory_structure() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create directory structure
    at.mkdir("testdir");
    at.write("testdir/file1.txt", "content1");
    at.mkdir("testdir/subdir");
    at.write("testdir/subdir/file2.txt", "content2");

    // Create archive
    ucmd.args(&["-cf", "archive.tar", "testdir"]).succeeds();

    // Remove directory contents and directory itself
    at.remove("testdir/subdir/file2.txt");
    at.remove("testdir/file1.txt");
    at.rmdir("testdir/subdir");
    at.rmdir("testdir");

    // Extract (extracts to current directory)
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify structure
    assert!(at.dir_exists("testdir"));
    assert!(at.file_exists("testdir/file1.txt"));
    assert!(at.dir_exists("testdir/subdir"));
    assert!(at.file_exists("testdir/subdir/file2.txt"));
    assert_eq!(at.read("testdir/file1.txt"), "content1");
    assert_eq!(at.read("testdir/subdir/file2.txt"), "content2");
}

// Round-trip tests

#[test]
fn test_roundtrip_empty_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create empty files
    at.touch("empty1.txt");
    at.touch("empty2.txt");

    // Create archive
    ucmd.args(&["-cf", "archive.tar", "empty1.txt", "empty2.txt"])
        .succeeds();

    // Remove originals
    at.remove("empty1.txt");
    at.remove("empty2.txt");

    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify empty files exist and are still empty
    assert!(at.file_exists("empty1.txt"));
    assert!(at.file_exists("empty2.txt"));
    assert_eq!(at.read("empty1.txt"), "");
    assert_eq!(at.read("empty2.txt"), "");
}

// Error handling and exit code tests

#[test]
#[cfg(unix)]
fn test_create_permission_denied() {
    // SAFETY: `libc::geteuid()` is a pure function that returns the effective
    // user ID of the calling process. It has no side effects and cannot cause
    // undefined behavior.
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipping: running as root");
        return;
    }

    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "content");
    at.mkdir("readonly");

    // Make directory read-only
    at.set_readonly("readonly");

    ucmd.args(&["-cf", "readonly/archive.tar", "file.txt"])
        .fails()
        .code_is(2)
        .stderr_contains("readonly/archive.tar");

    // Cleanup - restore permissions so test cleanup can work
    at.set_mode("readonly", 0o755);
}

#[test]
fn test_extract_corrupted_archive() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create a corrupted tar file (invalid header)
    at.write("corrupted.tar", "This is not a valid tar file content");

    ucmd.args(&["-xf", "corrupted.tar"]).fails().code_is(2);
}

#[test]
fn test_create_with_dash_in_filename() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create files starting with dash
    at.write("-dash-file.txt", "content with dash");
    at.write("normal.txt", "normal content");

    ucmd.args(&["-cf", "archive.tar", "--", "-dash-file.txt", "normal.txt"])
        .succeeds();

    assert!(at.file_exists("archive.tar"));

    // Verify extraction works
    at.remove("-dash-file.txt");
    at.remove("normal.txt");

    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("-dash-file.txt"));
    assert_eq!(at.read("-dash-file.txt"), "content with dash");
}

// CLI argument handling tests

#[test]
fn test_mixed_short_and_long_options() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "content");

    // Test mixing -x with --file
    ucmd.args(&["-c", "--file=archive.tar", "file.txt"])
        .succeeds();

    assert!(at.file_exists("archive.tar"));

    at.remove("file.txt");

    // Test extraction with mixed options
    new_ucmd!()
        .args(&["-x", "--file", "archive.tar"])
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("file.txt"));
}

#[test]
fn test_option_order_variations() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "content");

    // Test standard -cf order
    ucmd.args(&["-cf", "archive1.tar", "file.txt"]).succeeds();

    assert!(at.file_exists("archive1.tar"));

    // Test separate options
    new_ucmd!()
        .args(&["-c", "-f", "archive2.tar", "file.txt"])
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("archive2.tar"));

    // Test long form
    new_ucmd!()
        .args(&["--create", "--file", "archive3.tar", "file.txt"])
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("archive3.tar"));
}

#[test]
fn test_extract_overwrites_existing_by_default() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create original file and archive
    at.write("file.txt", "original content");
    ucmd.args(&["-cf", "archive.tar", "file.txt"]).succeeds();

    // Modify the file
    at.write("file.txt", "modified content");

    // Extract should overwrite
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify original content is restored
    assert_eq!(at.read("file.txt"), "original content");
}

// Edge case tests

// TODO(jeffbailey): This should move to tar-rs
#[test]
fn test_file_with_spaces_in_name() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create files with spaces in names
    at.write("file with spaces.txt", "content 1");
    at.write("another file.txt", "content 2");

    // Create archive
    ucmd.args(&[
        "-cf",
        "archive.tar",
        "file with spaces.txt",
        "another file.txt",
    ])
    .succeeds();

    // Remove originals
    at.remove("file with spaces.txt");
    at.remove("another file.txt");

    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify files extracted correctly
    assert!(at.file_exists("file with spaces.txt"));
    assert!(at.file_exists("another file.txt"));
    assert_eq!(at.read("file with spaces.txt"), "content 1");
    assert_eq!(at.read("another file.txt"), "content 2");
}

#[test]
fn test_large_number_of_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create 100 files
    let num_files = 100;
    for i in 0..num_files {
        at.write(&format!("file{i}.txt"), &format!("content {i}"));
    }

    // Collect file names for archive creation
    let files: Vec<String> = (0..num_files).map(|i| format!("file{i}.txt")).collect();
    let mut args = vec!["-cf", "archive.tar"];
    args.extend(files.iter().map(String::as_str));

    // Create archive
    ucmd.args(&args).succeeds();

    // Verify archive was created with reasonable size
    assert!(at.file_exists("archive.tar"));
    let archive_size = at.read_bytes("archive.tar").len();
    assert!(
        archive_size > TAR_BLOCK_SIZE * num_files,
        "Archive should contain data for {num_files} files"
    );
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
        .join(std::path::MAIN_SEPARATOR_STR);

    assert!(at.file_exists(expected_path));
}
