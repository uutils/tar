// This file is part of the uutils sed package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;
use clap::ArgMatches;
use sed::{get_scripts_files, ScriptValue};
use std::path::PathBuf;

// Test application's invocation
#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_debug() {
    new_ucmd!().arg("--debug").arg("").succeeds();
}

#[test]
fn test_silent_alias() {
    new_ucmd!().arg("--silent").arg("").succeeds();
}

#[test]
fn test_missing_script_argument() {
    new_ucmd!()
        .fails()
        .code_is(1)
        .stderr_contains("missing script");
}

#[test]
fn test_positional_script_ok() {
    new_ucmd!().arg("l").succeeds().code_is(0);
}

#[test]
fn test_e_script_ok() {
    new_ucmd!().arg("-e").arg("l").succeeds();
}

#[test]
fn test_f_script_ok() {
    new_ucmd!().arg("-f").arg("/dev/null").succeeds();
}

// Test the get_scripts_files function

// Helper function for supplying arguments
fn get_test_matches(args: &[&str]) -> ArgMatches {
    sed::uu_app().get_matches_from(["myapp"].iter().chain(args.iter()))
}

#[test]
fn test_script_as_first_argument() {
    let matches = get_test_matches(&["1d", "file1.txt"]);
    let (scripts, files) = get_scripts_files(&matches).expect("Should succeed");

    assert_eq!(scripts, vec![ScriptValue::StringVal("1d".to_string())]);
    assert_eq!(files, vec![PathBuf::from("file1.txt")]);
}

#[test]
fn test_expression_argument() {
    let matches = get_test_matches(&["-e", "s/foo/bar/", "file1.txt"]);
    let (scripts, files) = get_scripts_files(&matches).expect("Should succeed");

    assert_eq!(
        scripts,
        vec![ScriptValue::StringVal("s/foo/bar/".to_string())]
    );
    assert_eq!(files, vec![PathBuf::from("file1.txt")]);
}

#[test]
fn test_script_file_argument() {
    let matches = get_test_matches(&["-f", "script.sed", "file1.txt"]);
    let (scripts, files) = get_scripts_files(&matches).expect("Should succeed");

    assert_eq!(
        scripts,
        vec![ScriptValue::PathVal(PathBuf::from("script.sed"))]
    );
    assert_eq!(files, vec![PathBuf::from("file1.txt")]);
}

#[test]
fn test_multiple_files() {
    let matches = get_test_matches(&["-e", "s/foo/bar/", "file1.txt", "file2.txt"]);
    let (scripts, files) = get_scripts_files(&matches).expect("Should succeed");

    assert_eq!(
        scripts,
        vec![ScriptValue::StringVal("s/foo/bar/".to_string())]
    );
    assert_eq!(
        files,
        vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")]
    );
}

#[test]
fn test_multiple_files_script() {
    let matches = get_test_matches(&["s/foo/bar/", "file1.txt", "file2.txt"]);
    let (scripts, files) = get_scripts_files(&matches).expect("Should succeed");

    assert_eq!(
        scripts,
        vec![ScriptValue::StringVal("s/foo/bar/".to_string())]
    );
    assert_eq!(
        files,
        vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")]
    );
}

#[test]
fn test_stdin_when_no_files() {
    let matches = get_test_matches(&["-e", "s/foo/bar/"]);
    let (scripts, files) = get_scripts_files(&matches).expect("Should succeed");

    assert_eq!(
        scripts,
        vec![ScriptValue::StringVal("s/foo/bar/".to_string())]
    );
    assert_eq!(files, vec![PathBuf::from("-")]); // Stdin should be used
}

#[test]
fn test_stdin_when_no_files_script() {
    let matches = get_test_matches(&["s/foo/bar/"]);
    let (scripts, files) = get_scripts_files(&matches).expect("Should succeed");

    assert_eq!(
        scripts,
        vec![ScriptValue::StringVal("s/foo/bar/".to_string())]
    );
    assert_eq!(files, vec![PathBuf::from("-")]); // Stdin should be used
}
