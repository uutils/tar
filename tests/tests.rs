// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
use std::env;

pub const TESTS_BINARY: &str = env!("CARGO_BIN_EXE_tarapp");

// Use the ctor attribute to run this function before any tests
#[ctor::ctor]
fn init() {
    unsafe {
        // Necessary for uutests to be able to find the binary
        std::env::set_var("UUTESTS_BINARY_PATH", TESTS_BINARY);
    }
}

#[cfg(feature = "tar")]
#[path = "by-util/test_tar.rs"]
mod test_tar;
