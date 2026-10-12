// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;
use tar::uumain;

fn main() {
    let exit_code = uumain(env::args_os());

    // Only fatal errors are not recoverable: recovered ones set the exit code just
    // before a successful return (see `finish_after_previous_errors`).
    if exit_code == 2 && uucore::error::get_exit_code() != 2 {
        uucore::show_error!("Error is not recoverable: exiting now");
    }

    std::process::exit(exit_code);
}
