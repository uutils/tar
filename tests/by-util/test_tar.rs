// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uutests::new_ucmd;

// Basic tar tests
#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_help() {
    new_ucmd!().arg("--help").succeeds().code_is(0);
}

#[test]
fn test_version() {
    new_ucmd!().arg("--version").succeeds().code_is(0);
}
