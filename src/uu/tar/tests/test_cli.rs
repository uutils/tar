// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uu_tar::uu_app;

#[test]
fn test_extract_flag_parsing() {
    let app = uu_app();
    let result = app.try_get_matches_from(vec!["tar", "-xf", "archive.tar"]);
    assert!(result.is_ok());
    let matches = result.unwrap();
    assert!(matches.get_flag("extract"));
}

#[test]
fn test_create_flag_parsing() {
    let app = uu_app();
    let result = app.try_get_matches_from(vec!["tar", "-cf", "archive.tar", "file.txt"]);
    assert!(result.is_ok());
    let matches = result.unwrap();
    assert!(matches.get_flag("create"));
}

#[test]
fn test_verbose_flag_parsing() {
    let app = uu_app();
    let result = app.try_get_matches_from(vec!["tar", "-cvf", "archive.tar", "file.txt"]);
    assert!(result.is_ok());
    let matches = result.unwrap();
    assert!(matches.get_flag("verbose"));
    assert!(matches.get_flag("create"));
}
