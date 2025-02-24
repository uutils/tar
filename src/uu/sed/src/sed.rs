// This file is part of the uutils sed package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, Command};
use uucore::{error::UResult, format_usage};

const ABOUT: &str = "Stream editor for filtering and transforming text";
const USAGE: &str = "sed [-n] script [file...]";

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    // TODO remove underscore prefix when var is used
    let _matches = uu_app().try_get_matches_from(args)?;
    // TODO
    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([
            arg!(<script> "script to execute"),
            arg!([files] ... "input files"),
            arg!(-n --quiet "suppress automatic printing of pattern space"),
        ])
}
