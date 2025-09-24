// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;
use tar::uumain;

fn main() {
    std::process::exit(uumain(env::args_os()));
}
