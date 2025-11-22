// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;
use tar::uumain;

fn main() {
    let exit_code = uumain(env::args_os());

    // If exiting with code 2, show the "not recoverable" message
    if exit_code == 2 {
        uucore::show_error!("Error is not recoverable: exiting now");
    }

    std::process::exit(exit_code);
}
