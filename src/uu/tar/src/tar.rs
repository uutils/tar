// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod errors;
mod operations;

use clap::{arg, crate_version, ArgAction, Command};
use std::path::{Path, PathBuf};
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

    let verbose = matches.get_flag("verbose");

    // Handle extract operation
    if matches.get_flag("extract") {
        let archive_path = matches.get_one::<PathBuf>("file").ok_or_else(|| {
            uucore::error::USimpleError::new(1, "option requires an argument -- 'f'")
        })?;

        return operations::extract::extract_archive(archive_path, verbose);
    }

    // Handle create operation
    if matches.get_flag("create") {
        let archive_path = matches.get_one::<PathBuf>("file").ok_or_else(|| {
            uucore::error::USimpleError::new(1, "option requires an argument -- 'f'")
        })?;

        let files: Vec<&Path> = matches
            .get_many::<PathBuf>("files")
            .map(|v| v.map(|p| p.as_path()).collect())
            .unwrap_or_default();

        if files.is_empty() {
            return Err(uucore::error::USimpleError::new(
                1,
                "Cowardly refusing to create an empty archive",
            ));
        }

        return operations::create::create_archive(archive_path, &files, verbose);
    }

    // If no operation specified, show error
    Err(uucore::error::USimpleError::new(
        1,
        "You must specify one of the '-c' or '-x' options",
    ))
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
