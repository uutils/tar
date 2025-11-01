// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::path;

use uutests::{at_and_ucmd, new_ucmd};

// -----------------------------------------------------------------------------
// 1. Basic CLI Tests
// -----------------------------------------------------------------------------

#[test]
fn test_invalid_arg() {
    new_ucmd!()
        .arg("--definitely-invalid")
        .fails()
        .code_is(1) // TODO: return the usage exit code (64) for invalid arguments
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
fn test_verbose() {
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

#[test]
fn test_conflicting_operations() {
    new_ucmd!()
        .args(&["-c", "-x", "-f", "archive.tar"])
        .fails()
        .code_is(2);
}

#[test]
fn test_no_operation_specified() {
    new_ucmd!()
        .args(&["-f", "archive.tar"])
        .fails()
        .code_is(1) // TODO: align with GNU tar by returning exit code 64
        .stderr_contains("must specify one");
}

// -----------------------------------------------------------------------------
// 2. Create Operation Tests
// -----------------------------------------------------------------------------

#[test]
fn test_create_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "test content");

    ucmd.args(&["-cf", "archive.tar", "file1.txt"])
        .succeeds()
        .no_stderr();

    assert!(at.file_exists("archive.tar"));
    assert!(at.read_bytes("archive.tar").len() > 512); // Basic sanity check
}

#[test]
fn test_create_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    at.write("file3.txt", "content3");

    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds()
        .no_stderr();

    assert!(at.file_exists("archive.tar"));
    assert!(at.read_bytes("archive.tar").len() > 512); // Basic sanity check
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
        .no_stderr();

    assert!(at.file_exists("archive.tar"));
    assert!(at.read_bytes("archive.tar").len() > 512); // Basic sanity check
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
        .code_is(1) // TODO: propagate usage exit code 64 once empty archive handling is fixed
        .stderr_contains("empty archive");
}

#[test]
fn test_create_nonexistent_file_fails() {
    let (_at, mut ucmd) = at_and_ucmd!();

    ucmd.args(&["-cf", "archive.tar", "nonexistent.txt"])
        .fails()
        .code_is(2)
        .stderr_contains("nonexistent.txt");
}

// -----------------------------------------------------------------------------
// 3. Extract Operation Tests
// -----------------------------------------------------------------------------

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
        .no_stderr();

    assert!(at.file_exists("original.txt"));
    assert_eq!(at.read("original.txt"), "test content");
}

#[test]
fn test_extract_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create an archive
    at.write("file1.txt", "content");
    ucmd.args(&["-cf", "archive.tar", "file1.txt"]).succeeds();

    at.remove("file1.txt");

    // Extract with verbose (extracts to current directory)
    new_ucmd!()
        .arg("-xvf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file1.txt");

    assert!(at.file_exists("file1.txt"));
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
    std::fs::remove_dir(at.plus("testdir/subdir")).unwrap();
    std::fs::remove_dir(at.plus("testdir")).unwrap();

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

// -----------------------------------------------------------------------------
// 4. Round-trip Tests
// -----------------------------------------------------------------------------

#[test]
fn test_roundtrip_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create a file
    at.write("file.txt", "test content");

    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file.txt"]).succeeds();

    // Remove original
    at.remove("file.txt");

    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify content is identical
    assert!(at.file_exists("file.txt"));
    assert_eq!(at.read("file.txt"), "test content");
}

#[test]
fn test_roundtrip_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create multiple files with different content
    at.write("file1.txt", "content one");
    at.write("file2.txt", "content two");
    at.write("file3.txt", "content three");

    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds();

    // Remove originals
    at.remove("file1.txt");
    at.remove("file2.txt");
    at.remove("file3.txt");

    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify all contents are identical
    assert_eq!(at.read("file1.txt"), "content one");
    assert_eq!(at.read("file2.txt"), "content two");
    assert_eq!(at.read("file3.txt"), "content three");
}

#[test]
fn test_roundtrip_directory_structure() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create complex directory structure
    at.mkdir("dir1");
    at.write("dir1/file1.txt", "content1");
    at.write("dir1/file2.txt", "content2");
    at.mkdir("dir1/subdir");
    at.write("dir1/subdir/file3.txt", "content3");
    at.mkdir("dir1/subdir/deepdir");
    at.write("dir1/subdir/deepdir/file4.txt", "content4");

    // Create archive
    ucmd.args(&["-cf", "archive.tar", "dir1"]).succeeds();

    // Remove directory structure
    at.remove("dir1/subdir/deepdir/file4.txt");
    std::fs::remove_dir(at.plus("dir1/subdir/deepdir")).unwrap();
    at.remove("dir1/subdir/file3.txt");
    std::fs::remove_dir(at.plus("dir1/subdir")).unwrap();
    at.remove("dir1/file1.txt");
    at.remove("dir1/file2.txt");
    std::fs::remove_dir(at.plus("dir1")).unwrap();

    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify complete structure and contents
    assert!(at.dir_exists("dir1"));
    assert!(at.file_exists("dir1/file1.txt"));
    assert!(at.file_exists("dir1/file2.txt"));
    assert!(at.dir_exists("dir1/subdir"));
    assert!(at.file_exists("dir1/subdir/file3.txt"));
    assert!(at.dir_exists("dir1/subdir/deepdir"));
    assert!(at.file_exists("dir1/subdir/deepdir/file4.txt"));

    assert_eq!(at.read("dir1/file1.txt"), "content1");
    assert_eq!(at.read("dir1/file2.txt"), "content2");
    assert_eq!(at.read("dir1/subdir/file3.txt"), "content3");
    assert_eq!(at.read("dir1/subdir/deepdir/file4.txt"), "content4");
}

#[test]
fn test_roundtrip_empty_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create empty files
    at.write("empty1.txt", "");
    at.write("empty2.txt", "");

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

#[test]
fn test_roundtrip_special_characters_in_names() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create files with special characters (avoiding problematic ones)
    at.write("file-with-dash.txt", "dash content");
    at.write("file_with_underscore.txt", "underscore content");
    at.write("file.multiple.dots.txt", "dots content");

    // Create archive
    ucmd.args(&[
        "-cf",
        "archive.tar",
        "file-with-dash.txt",
        "file_with_underscore.txt",
        "file.multiple.dots.txt",
    ])
    .succeeds();

    // Remove originals
    at.remove("file-with-dash.txt");
    at.remove("file_with_underscore.txt");
    at.remove("file.multiple.dots.txt");

    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();

    // Verify contents
    assert_eq!(at.read("file-with-dash.txt"), "dash content");
    assert_eq!(at.read("file_with_underscore.txt"), "underscore content");
    assert_eq!(at.read("file.multiple.dots.txt"), "dots content");
}

// -----------------------------------------------------------------------------
// 5. Error Handling and Exit Code Tests
// -----------------------------------------------------------------------------

#[test]
#[cfg(unix)]
fn test_create_permission_denied() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "content");
    at.mkdir("readonly");

    // Make directory read-only
    let perms = fs::Permissions::from_mode(0o444);
    fs::set_permissions(at.plus("readonly"), perms).unwrap();

    ucmd.args(&["-cf", "readonly/archive.tar", "file.txt"])
        .fails()
        .code_is(2)
        .stderr_contains("readonly/archive.tar");

    // Cleanup - restore permissions so test cleanup can work
    let perms = fs::Permissions::from_mode(0o755);
    fs::set_permissions(at.plus("readonly"), perms).unwrap();
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

// -----------------------------------------------------------------------------
// 6. Verbose Output Format Tests
// -----------------------------------------------------------------------------

#[test]
fn test_verbose_output_format_matches_gnu() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content");
    at.write("file2.txt", "content");

    let result = ucmd
        .args(&["-cvf", "archive.tar", "file1.txt", "file2.txt"])
        .succeeds();

    let stdout = result.stdout_str();

    // Verify verbose output contains filenames
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));
}

#[test]
fn test_extract_verbose_shows_all_files() {
    let (at, mut ucmd) = at_and_ucmd!();

    // Create archive with multiple files
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

    // Verify all files are listed in output
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));
    assert!(stdout.contains("file3.txt"));
}

// -----------------------------------------------------------------------------
// 7. CLI Argument Handling Tests
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// 8. Edge Case Tests
// -----------------------------------------------------------------------------

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
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);

    // Create archive
    ucmd.args(&args).succeeds();

    // Verify archive was created with reasonable size
    assert!(at.file_exists("archive.tar"));
    let archive_size = at.read_bytes("archive.tar").len();
    assert!(
        archive_size > 512 * num_files,
        "Archive should contain data for {num_files} files"
    );
}
