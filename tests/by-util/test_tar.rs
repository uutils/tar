// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::io::{Cursor, Read, Write};
use std::path::{self, PathBuf};

use tar_rs_crate::{Archive as TarRsArchive, Builder as TarRsBuilder, Header as TarRsHeader};
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
#[test]
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
fn test_create_zstd_archive() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "test content");

    ucmd.args(&["--zstd", "-cf", "archive.tar.zst", "file1.txt"])
        .succeeds()
        .no_output();

    assert!(at.file_exists("archive.tar.zst"));
    assert!(!at.read_bytes("archive.tar.zst").is_empty());
}

#[test]
fn test_create_zstd_archive_is_readable_by_independent_readers() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "test content");

    ucmd.args(&["--zstd", "-cf", "archive.tar.zst", "file1.txt"])
        .succeeds()
        .no_output();

    let compressed = at.read_bytes("archive.tar.zst");
    let decoded = zstd::stream::decode_all(Cursor::new(compressed)).unwrap();
    let mut archive = TarRsArchive::new(Cursor::new(decoded));
    let mut entries = archive.entries().unwrap();
    let mut entry = entries.next().unwrap().unwrap();

    assert_eq!(entry.path().unwrap().to_str(), Some("file1.txt"));

    let mut contents = String::new();
    entry.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "test content");
    assert!(entries.next().is_none());
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
fn test_create_verbose_to_stdout_uses_stderr() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "hello");

    ucmd.args(&["-cvf", "-", "file.txt"])
        .succeeds()
        .stderr_contains("file.txt");
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

    // Trim leading '/'
    ucmd.args(&[
        "-cf",
        "archive-trimed.tar",
        &file_abs_path.display().to_string(),
    ])
    .succeeds()
    .stderr_contains("tar: Removing leading");

    assert!(at.file_exists("archive-trimed.tar"));

    let expected_trimmed_path = file_abs_path
        .components()
        .filter(|c| !matches!(c, path::Component::RootDir | path::Component::Prefix(_)))
        .map(|c| c.as_os_str().display().to_string())
        .collect::<Vec<_>>()
        .join("/");

    new_ucmd!()
        .args(&["-tf", "archive-trimed.tar"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains(&expected_trimmed_path);

    let (at, mut ucmd) = at_and_ucmd!();
    // Preserve leading '/'
    ucmd.args(&[
        "-cPf",
        "archive-preserved.tar",
        &file_abs_path.display().to_string(),
    ])
    .succeeds()
    .no_output();

    assert!(at.file_exists("archive-preserved.tar"));

    let expected_prefix = file_abs_path
        .components()
        .find(|c| matches!(c, path::Component::RootDir | path::Component::Prefix(_)))
        .map(|c| c.as_os_str().display().to_string())
        .unwrap();

    new_ucmd!()
        .args(&["-tf", "archive-preserved.tar"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains(expected_prefix)
        .stdout_contains(expected_trimmed_path);
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
fn test_list_zstd_archive() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");

    ucmd.args(&["--zstd", "-cf", "archive.tar.zst", "file1.txt", "file2.txt"])
        .succeeds();

    new_ucmd!()
        .args(&["--zstd", "-tf", "archive.tar.zst"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file1.txt")
        .stdout_contains("file2.txt");
}

#[test]
fn test_list_zstd_archive_created_outside_tar() {
    let at = &at_and_ucmd!().0;

    let mut tar_bytes = Vec::new();
    {
        let mut builder = TarRsBuilder::new(&mut tar_bytes);
        let mut header = TarRsHeader::new_gnu();
        header.set_mode(0o644);
        header.set_size("content".len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, "external.txt", Cursor::new("content"))
            .unwrap();
        builder.finish().unwrap();
    }

    let compressed = zstd::stream::encode_all(Cursor::new(tar_bytes), 0).unwrap();
    at.write_bytes("external.tar.zst", &compressed);

    new_ucmd!()
        .args(&["--zstd", "-tf", "external.tar.zst"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("external.txt");
}

#[test]
fn test_extract_zstd_archive() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("original.txt", "test content");
    ucmd.args(&["--zstd", "-cf", "archive.tar.zst", "original.txt"])
        .succeeds();

    at.remove("original.txt");

    new_ucmd!()
        .args(&["--zstd", "-xf", "archive.tar.zst"])
        .current_dir(at.as_string())
        .succeeds()
        .no_output();

    assert!(at.file_exists("original.txt"));
    assert_eq!(at.read("original.txt"), "test content");
}

#[test]
fn test_list_zstd_archive_without_flag_fails() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content1");
    ucmd.args(&["--zstd", "-cf", "archive.tar.zst", "file1.txt"])
        .succeeds();

    new_ucmd!()
        .args(&["-tf", "archive.tar.zst"])
        .current_dir(at.as_string())
        .fails()
        .code_is(2);
}

#[test]
fn test_list_invalid_zstd_archive_fails() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("invalid.tar.zst", "definitely not zstd");

    ucmd.args(&["--zstd", "-tf", "invalid.tar.zst"])
        .fails()
        .code_is(2);
}

#[test]
fn test_list_truncated_zstd_archive_fails() {
    let at = &at_and_ucmd!().0;

    let mut tar_bytes = Vec::new();
    {
        let mut builder = TarRsBuilder::new(&mut tar_bytes);
        let mut header = TarRsHeader::new_gnu();
        header.set_mode(0o644);
        header.set_size("content".len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, "truncated.txt", Cursor::new("content"))
            .unwrap();
        builder.finish().unwrap();
    }

    let mut compressed = zstd::stream::encode_all(Cursor::new(tar_bytes), 0).unwrap();
    compressed.truncate(compressed.len().saturating_sub(2));
    at.write_bytes("truncated.tar.zst", &compressed);

    new_ucmd!()
        .args(&["--zstd", "-tf", "truncated.tar.zst"])
        .current_dir(at.as_string())
        .fails()
        .code_is(2);
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
    if rustix::process::geteuid().is_root() {
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
    ucmd.args(&[
        "-cf",
        "archive-trimed.tar",
        &file_abs_path.display().to_string(),
    ])
    .succeeds();

    new_ucmd!()
        .args(&["-xf", "archive-trimed.tar"])
        .current_dir(at.as_string())
        .succeeds();

    let expected_path = file_abs_path
        .components()
        .filter(|c| !matches!(c, path::Component::RootDir | path::Component::Prefix(_)))
        .map(|c| c.as_os_str().display().to_string())
        .collect::<Vec<_>>()
        .join(std::path::MAIN_SEPARATOR_STR);

    assert!(at.file_exists(&expected_path));

    new_ucmd!()
        .args(&[
            "-cPf",
            "archive-preserved.tar",
            &file_abs_path.display().to_string(),
        ])
        .current_dir(at.as_string())
        .succeeds();

    at.remove(&expected_path);
    new_ucmd!()
        .args(&["-xf", "archive-preserved.tar"])
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists(expected_path));
}

// POSIX keystring tests (no leading '-' on the key operand)

#[test]
fn test_posix_create_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content");

    ucmd.args(&["cvf", "archive.tar", "file1.txt"])
        .succeeds()
        .stdout_contains("file1.txt");

    assert!(at.file_exists("archive.tar"));
}

#[test]
fn test_posix_extract_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    ucmd.args(&["cf", "archive.tar", "file1.txt", "file2.txt"])
        .succeeds();

    at.remove("file1.txt");
    at.remove("file2.txt");

    let result = new_ucmd!()
        .args(&["xvf", &at.plus_as_string("archive.tar")])
        .current_dir(at.as_string())
        .succeeds();

    let stdout = result.stdout_str();
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));

    assert!(at.file_exists("file1.txt"));
    assert!(at.file_exists("file2.txt"));
}

#[test]
fn test_posix_and_dash_prefix_both_work() {
    // Confirm that POSIX-style and dash-prefixed styles produce identical results.
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "hello");

    // POSIX style
    ucmd.args(&["cf", "posix.tar", "file.txt"]).succeeds();

    // Dash-prefix style
    new_ucmd!()
        .args(&["-cf", "dash.tar", "file.txt"])
        .current_dir(at.as_string())
        .succeeds();

    assert_eq!(
        at.read_bytes("posix.tar").len(),
        at.read_bytes("dash.tar").len()
    );
}

#[test]
fn test_posix_b_matches_dash_prefix_failure() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "hello");

    // Dash-prefixed form currently rejects '-b'.
    new_ucmd!()
        .args(&["-cbf", "20", "dash.tar", "file.txt"])
        .current_dir(at.as_string())
        .fails()
        .code_is(64)
        .stderr_contains("unexpected argument '-b'");

    // POSIX keystring form should fail with the same unsupported option.
    ucmd.args(&["cbf", "20", "posix.tar", "file.txt"])
        .fails()
        .code_is(64)
        .stderr_contains("unexpected argument '-b'");
}

// List operation tests

#[test]
fn test_list_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("file.txt", "test content");
    ucmd.args(&["-cf", "archive.tar", "file.txt"]).succeeds();

    new_ucmd!()
        .args(&["-tf", "archive.tar"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file.txt");
}

#[test]
fn test_list_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    at.write("file3.txt", "content3");

    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds();

    new_ucmd!()
        .args(&["-tf", "archive.tar"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file1.txt")
        .stdout_contains("file2.txt")
        .stdout_contains("file3.txt");
}

#[test]
fn test_list_directory() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.mkdir("testdir");
    at.write("testdir/file1.txt", "content1");
    at.mkdir("testdir/subdir");
    at.write("testdir/subdir/file2.txt", "content2");

    ucmd.args(&["-cf", "archive.tar", "testdir"]).succeeds();

    new_ucmd!()
        .args(&["-tf", "archive.tar"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("testdir")
        .stdout_contains("testdir/file1.txt")
        .stdout_contains("testdir/subdir")
        .stdout_contains("testdir/subdir/file2.txt");
}

#[test]
fn test_list_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("file.txt", "content");
    ucmd.args(&["-cf", "archive.tar", "file.txt"]).succeeds();

    new_ucmd!()
        .args(&["-tvf", "archive.tar"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file.txt")
        .stdout_contains("7 "); // verbose output includes file size; absent from plain -t listing
}

#[test]
fn test_list_nonexistent_archive() {
    new_ucmd!()
        .args(&["-tf", "nonexistent.tar"])
        .fails()
        .code_is(2)
        .stderr_contains("nonexistent.tar");
}

#[test]
fn test_list_conflicts_with_create() {
    new_ucmd!()
        .args(&["-ctf", "archive.tar", "file.txt"])
        .fails()
        .code_is(2)
        .stderr_contains("cannot be used with");
}

#[test]
fn test_list_conflicts_with_extract() {
    new_ucmd!()
        .args(&["-xtf", "archive.tar"])
        .fails()
        .code_is(2)
        .stderr_contains("cannot be used with");
}

// stdin/stdout (-f -) tests

#[test]
fn test_create_to_stdout() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("file.txt", "hello");

    let result = ucmd.args(&["-cf", "-", "file.txt"]).succeeds();
    let bytes = result.stdout();
    // A tar archive is at least two blocks: one header + two end-of-archive blocks
    assert!(
        bytes.len() >= TAR_BLOCK_SIZE,
        "stdout should contain tar data (got {} bytes)",
        bytes.len()
    );
    // The first 100 bytes of a tar header are the file name
    let header_prefix = std::str::from_utf8(&bytes[..100]).unwrap_or("");
    assert!(
        header_prefix.contains("file.txt"),
        "tar header should contain filename"
    );
}

#[test]
fn test_create_verbose_to_stdout_keeps_stdout_as_tar_data() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("file.txt", "hello");

    let result = ucmd.args(&["-cvf", "-", "file.txt"]).succeeds();
    let bytes = result.stdout();
    let header_prefix = std::str::from_utf8(&bytes[..100]).unwrap_or("");

    assert!(
        bytes.len() >= TAR_BLOCK_SIZE,
        "stdout should contain tar data (got {} bytes)",
        bytes.len()
    );
    assert!(
        header_prefix.contains("file.txt"),
        "tar header should contain filename"
    );
    assert_eq!(&bytes[..8], b"file.txt");
    assert_eq!(
        bytes[8], 0,
        "stdout should not contain verbose output prefix"
    );
}

#[test]
fn test_create_absolute_path_to_stdout_uses_stderr_for_normalization_notice() {
    let (at, mut ucmd) = at_and_ucmd!();

    let mut file_abs_path = PathBuf::from(at.root_dir_resolved());
    file_abs_path.push("file1.txt");

    at.write(&file_abs_path.display().to_string(), "content1");

    let result = ucmd
        .args(&["-cf", "-", &file_abs_path.display().to_string()])
        .succeeds();
    result.stderr_contains("Removing leading");
    let bytes = result.stdout();
    let header_prefix = std::str::from_utf8(&bytes[..100]).unwrap_or("");

    assert!(
        bytes.len() >= TAR_BLOCK_SIZE,
        "stdout should contain tar data (got {} bytes)",
        bytes.len()
    );
    assert!(
        !header_prefix.contains("Removing leading"),
        "stdout should not contain normalization notices"
    );
}

#[test]
fn test_list_from_stdin() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("file.txt", "hello");

    // Create archive to a file in the same sandbox
    new_ucmd!()
        .args(&["-cf", &at.plus_as_string("archive.tar"), "file.txt"])
        .current_dir(at.as_string())
        .succeeds();

    let archive_bytes = at.read_bytes("archive.tar");
    ucmd.args(&["-tf", "-"])
        .pipe_in(archive_bytes)
        .succeeds()
        .stdout_contains("file.txt");
}

#[test]
fn test_extract_from_stdin() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("file.txt", "hello world");

    // Create archive to a file in the same sandbox
    new_ucmd!()
        .args(&["-cf", &at.plus_as_string("archive.tar"), "file.txt"])
        .current_dir(at.as_string())
        .succeeds();

    at.remove("file.txt");

    let archive_bytes = at.read_bytes("archive.tar");
    ucmd.args(&["-xf", "-"]).pipe_in(archive_bytes).succeeds();

    assert_eq!(at.read("file.txt"), "hello world");
}

#[test]
fn test_gzip_verbose_stdio_roundtrip() {
    let (at, mut ucmd) = at_and_ucmd!();
    at.write("stdio-gzip.txt", "gzip through stdio");

    let result = ucmd.args(&["-zcvf", "-", "stdio-gzip.txt"]).succeeds();
    result.stderr_contains("stdio-gzip.txt");
    let archive_bytes = result.stdout().to_vec();
    assert_eq!(&archive_bytes[..2], &[0x1f, 0x8b]);

    new_ucmd!()
        .args(&["-tf", "-"])
        .pipe_in(archive_bytes.clone())
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("stdio-gzip.txt");

    at.remove("stdio-gzip.txt");

    new_ucmd!()
        .args(&["-xf", "-"])
        .pipe_in(archive_bytes)
        .current_dir(at.as_string())
        .succeeds();

    assert_eq!(at.read("stdio-gzip.txt"), "gzip through stdio");
}

// Gzip-compressed archive tests

#[test]
fn test_extract_gzip_archive() {
    let (at, _ucmd) = at_and_ucmd!();

    // Build a .tar.gz in memory: tar containing one file, then gzip-compress it
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar_rs_crate::Builder::new(&mut tar_bytes);
        let content = b"hello from gzip";
        let mut header = tar_rs_crate::Header::new_gnu();
        header.set_path("gzfile.txt").unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &content[..]).unwrap();
        builder.finish().unwrap();
    }

    // Gzip-compress the tar bytes
    let mut gz_bytes = Vec::new();
    {
        let mut encoder =
            flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap();
    }

    at.write_bytes("archive.tar.gz", &gz_bytes);

    // Extract using our tar implementation
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar.gz"))
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("gzfile.txt"));
    assert_eq!(at.read("gzfile.txt"), "hello from gzip");
}

#[test]
fn test_extract_gzip_archive_with_explicit_flag() {
    let (at, _ucmd) = at_and_ucmd!();

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar_rs_crate::Builder::new(&mut tar_bytes);
        let content = b"hello from explicit gzip";
        let mut header = tar_rs_crate::Header::new_gnu();
        header.set_path("explicit-gzfile.txt").unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &content[..]).unwrap();
        builder.finish().unwrap();
    }

    let mut gz_bytes = Vec::new();
    {
        let mut encoder =
            flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap();
    }

    at.write_bytes("archive.tar.gz", &gz_bytes);

    new_ucmd!()
        .args(&["-zxf", &at.plus_as_string("archive.tar.gz")])
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("explicit-gzfile.txt"));
    assert_eq!(at.read("explicit-gzfile.txt"), "hello from explicit gzip");
}

#[test]
fn test_create_gzip_archive() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "test content");

    ucmd.args(&["-zcf", "archive.tar.gz", "file1.txt"])
        .succeeds()
        .no_output();

    let mut decoder =
        flate2::read::GzDecoder::new(std::io::Cursor::new(at.read_bytes("archive.tar.gz")));
    let mut decoded = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut decoded).unwrap();

    let mut archive = tar_rs_crate::Archive::new(std::io::Cursor::new(decoded));
    let mut entries = archive.entries().unwrap();
    let mut entry = entries.next().unwrap().unwrap();
    let mut contents = String::new();
    std::io::Read::read_to_string(&mut entry, &mut contents).unwrap();

    assert_eq!(entry.path().unwrap().to_str(), Some("file1.txt"));
    assert_eq!(contents, "test content");
    assert!(entries.next().is_none());
}

#[test]
fn test_gzip_roundtrip() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("roundtrip.txt", "roundtrip gzip content");

    ucmd.args(&["-zcf", "archive.tar.gz", "roundtrip.txt"])
        .succeeds()
        .no_output();

    at.remove("roundtrip.txt");

    new_ucmd!()
        .args(&["-xf", &at.plus_as_string("archive.tar.gz")])
        .current_dir(at.as_string())
        .succeeds()
        .no_output();

    assert!(at.file_exists("roundtrip.txt"));
    assert_eq!(at.read("roundtrip.txt"), "roundtrip gzip content");
}

#[test]
fn test_list_gzip_archive() {
    let (at, _ucmd) = at_and_ucmd!();

    // Build a .tar.gz in memory
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar_rs_crate::Builder::new(&mut tar_bytes);
        let content = b"list test content";
        let mut header = tar_rs_crate::Header::new_gnu();
        header.set_path("listed.txt").unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &content[..]).unwrap();
        builder.finish().unwrap();
    }

    let mut gz_bytes = Vec::new();
    {
        let mut encoder =
            flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap();
    }

    at.write_bytes("archive.tar.gz", &gz_bytes);

    new_ucmd!()
        .args(&["-tf", &at.plus_as_string("archive.tar.gz")])
        .succeeds()
        .stdout_contains("listed.txt");
}

#[test]
fn test_list_gzip_archive_with_explicit_flag() {
    let (at, _ucmd) = at_and_ucmd!();

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar_rs_crate::Builder::new(&mut tar_bytes);
        let content = b"explicit gzip";
        let mut header = tar_rs_crate::Header::new_gnu();
        header.set_path("explicit.txt").unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &content[..]).unwrap();
        builder.finish().unwrap();
    }

    let mut gz_bytes = Vec::new();
    {
        let mut encoder =
            flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap();
    }

    at.write_bytes("archive.tar.gz", &gz_bytes);

    new_ucmd!()
        .args(&["-ztf", &at.plus_as_string("archive.tar.gz")])
        .succeeds()
        .stdout_contains("explicit.txt");
}

#[test]
fn test_extract_invalid_gzip_archive_fails() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("invalid.tar.gz", "definitely not gzip");

    ucmd.args(&["-xf", "invalid.tar.gz"]).fails().code_is(2);
}

// --wildcards tests

#[test]
fn test_list_wildcards_star_pattern() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "txt");
    at.write("file.rs", "rs");

    ucmd.args(&["-cf", "archive.tar", "file.txt", "file.rs"])
        .succeeds();

    new_ucmd!()
        .args(&["-tf", "archive.tar", "--wildcards", "*.txt"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file.txt")
        .stdout_does_not_contain("file.rs");
}

#[test]
fn test_list_wildcards_question_mark_pattern() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file1.txt", "1");
    at.write("file2.txt", "2");
    at.write("file10.txt", "10");

    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file10.txt"])
        .succeeds();

    new_ucmd!()
        .args(&["-tf", "archive.tar", "--wildcards", "file?.txt"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file1.txt")
        .stdout_contains("file2.txt")
        .stdout_does_not_contain("file10.txt");
}

#[test]
fn test_extract_wildcards() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("keep.txt", "keep");
    at.write("skip.rs", "skip");

    ucmd.args(&["-cf", "archive.tar", "keep.txt", "skip.rs"])
        .succeeds();

    at.remove("keep.txt");
    at.remove("skip.rs");

    new_ucmd!()
        .args(&["-xf", "archive.tar", "--wildcards", "*.txt"])
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("keep.txt"));
    assert!(!at.file_exists("skip.rs"));
}

#[test]
fn test_list_no_wildcards_exact_match() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("file.txt", "a");
    at.write("other.txt", "b");

    ucmd.args(&["-cf", "archive.tar", "file.txt", "other.txt"])
        .succeeds();

    // Without --wildcards, "*.txt" should match nothing (literal string)
    new_ucmd!()
        .args(&["-tf", "archive.tar", "*.txt"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_does_not_contain("file.txt")
        .stdout_does_not_contain("other.txt");
}

// --strip-components tests

#[test]
fn test_list_strip_components() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.mkdir("dir");
    at.write("dir/file.txt", "content");

    ucmd.args(&["-cf", "archive.tar", "dir"]).succeeds();

    new_ucmd!()
        .args(&["-tf", "archive.tar", "--strip-components=1"])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file.txt")
        .stdout_does_not_contain("dir/file.txt");
}

#[test]
fn test_extract_strip_components() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.mkdir("dir");
    at.write("dir/file.txt", "content");

    ucmd.args(&["-cf", "archive.tar", "dir"]).succeeds();

    at.remove("dir/file.txt");

    new_ucmd!()
        .args(&["-xf", "archive.tar", "--strip-components=1"])
        .current_dir(at.as_string())
        .succeeds();

    assert!(at.file_exists("file.txt"));
    assert_eq!(at.read("file.txt"), "content");
}

#[test]
fn test_extract_strip_components_skips_shallow_entries() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.write("shallow.txt", "shallow");
    at.mkdir("dir");
    at.write("dir/deep.txt", "deep");

    ucmd.args(&["-cf", "archive.tar", "shallow.txt", "dir"])
        .succeeds();

    at.remove("shallow.txt");
    at.remove("dir/deep.txt");

    new_ucmd!()
        .args(&["-xf", "archive.tar", "--strip-components=1"])
        .current_dir(at.as_string())
        .succeeds();

    // shallow.txt has only one component; stripping 1 discards it
    assert!(!at.file_exists("shallow.txt"));
    assert!(at.file_exists("deep.txt"));
}

#[test]
fn test_wildcards_and_strip_components_combined() {
    let (at, mut ucmd) = at_and_ucmd!();

    at.mkdir("dir");
    at.write("dir/keep.txt", "keep");
    at.write("dir/skip.rs", "skip");

    ucmd.args(&["-cf", "archive.tar", "dir"]).succeeds();

    new_ucmd!()
        .args(&[
            "-tf",
            "archive.tar",
            "--wildcards",
            "--strip-components=1",
            "*.txt",
        ])
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("keep.txt")
        .stdout_does_not_contain("skip.rs");
}
