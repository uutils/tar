// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod errors;
mod operations;
mod options;

use clap::{arg, crate_version, ArgAction, Command};
use operations::operation::TarOperation;
use options::TarParams;
use std::path::PathBuf;
use uucore::error::UResult;
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {c|x}[v] -f ARCHIVE [FILE...]";

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    // Collect args - the test framework may add util_name as args[1], so skip it if present
    let args_vec: Vec<_> = args.collect();
    let util_name = uucore::util_name();

    // Skip duplicate util name if present (can be "tar" or "tarapp")
    let args_to_parse = if args_vec.len() > 1
        && (args_vec[1] == util_name || args_vec[1] == "tar" || args_vec[1] == "tarapp")
    {
        let mut result = vec![args_vec[0].clone()];
        result.extend_from_slice(&args_vec[2..]);
        result
    } else {
        args_vec
    };

    let matches = uu_app().try_get_matches_from(args_to_parse)?;

    let (op, options) = TarParams::with_operation(&matches)?;

    op.exec(&options)
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .args([
            // Main operation modes
            arg!(-c --create "Create a new archive"),
            // arg!(-d --diff "Find differences between archive and file system").alias("compare"),
            // arg!(-r --append "Append files to end of archive"),
            // arg!(-t --list "List contents of archive"),
            // arg!(-u --update "Only append files newer than copy in archive"),
            arg!(-x --extract "Extract files from archive").alias("get"),
            // Archive file
            arg!(-f --file <ARCHIVE> "Use archive file or device ARCHIVE")
                .value_parser(clap::value_parser!(PathBuf)),
            // Compression options
            // arg!(-z --gzip "Filter through gzip"),
            // arg!(-j --bzip2 "Filter through bzip2"),
            // arg!(-J --xz "Filter through xz"),
            // Common options
            arg!(-v --verbose "Verbosely list files processed"),
            // arg!(-h --dereference "Follow symlinks"),
            // arg!(-p --"preserve-permissions" "Extract information about file permissions"),
            // arg!(-P --"absolute-names" "Don't strip leading '/' from file names"),
            // Help
            arg!(--help "Print help information").action(ArgAction::Help),
            // Files to process
            arg!([files]... "Files to archive or extract")
                .action(ArgAction::Append)
                .value_parser(clap::value_parser!(PathBuf)),
        ])
}
