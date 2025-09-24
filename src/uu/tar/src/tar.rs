// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, Arg, Command};
use std::path::PathBuf;
use uucore::error::UResult;
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] [ARG...]";

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // For now, just print a basic message indicating the command was parsed
    println!("tar: basic implementation - command line parsed successfully");

    // Check if any files were specified
    if let Some(files) = matches.get_many::<PathBuf>("file") {
        for file in files {
            println!("File: {}", file.display());
        }
    }

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .args([
            // Main operation modes
            arg!(-A --catenate "Append tar files to archive"),
            arg!(-c --create "Create a new archive"),
            arg!(-d --diff "Find differences between archive and file system").alias("compare"),
            arg!(-r --append "Append files to end of archive"),
            arg!(-t --list "List contents of archive"),
            arg!(-u --update "Only append files newer than copy in archive"),
            arg!(-x --extract "Extract files from archive").alias("get"),
            // Archive file
            arg!(-f --file <ARCHIVE> "Use archive file").value_parser(clap::value_parser!(PathBuf)),
            // Compression options
            arg!(-z --gzip "Filter through gzip"),
            arg!(-j --bzip2 "Filter through bzip2"),
            arg!(-J --xz "Filter through xz"),
            // Common options
            arg!(-v --verbose "Verbosely list files processed"),
            arg!(-h --dereference "Follow symlinks"),
            arg!(-p --preserve-permissions "Extract information about file permissions"),
            arg!(-P --absolute-names "Don't strip leading '/' from file names"),
            // Files to process
            Arg::new("file")
                .help("Files to archive or extract")
                .value_parser(clap::value_parser!(PathBuf))
                .num_args(0..),
        ])
}

#[cfg(test)]
mod tests {}
