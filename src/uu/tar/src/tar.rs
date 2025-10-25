// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
mod archive;
mod list;
mod operation;
mod options;
mod util;

use crate::operation::TarOperation;
use crate::options::*;
use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, Command};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use uucore::error::{UError, UResult};
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] [ARG...]";

// TODO: get rid of the NotGood error across all files
// TODO: add more descriptive situational errors
pub enum TarError {
    NotGood,
    InvalidMagic,
    InvalidOperation(String),
    ParseError,
}

impl Display for TarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TarError::NotGood => f.write_str("TarError: Not Good...This will be replaced later"),
            TarError::InvalidMagic => f.write_str("TarError: Invalid Magic"),
            TarError::InvalidOperation(m) => {
                f.write_str(&format!("TarError: Invalid Operation: {}", m))
            }
            TarError::ParseError => f.write_str("TarError: ParseError"),
        }
    }
}
impl Debug for TarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TarError::NotGood => f.write_str("TarError: Not Good...This will be replaced later"),
            TarError::InvalidMagic => f.write_str("TarError: Invalid Magic"),
            TarError::InvalidOperation(m) => {
                f.write_str(&format!("TarError: Invalid Operation: {}", m))
            }
            TarError::ParseError => f.write_str("TarError: ParseError"),
        }
    }
}

// TODO: give some actual error handling here
// when coming from a formatting error
impl From<std::fmt::Error> for TarError {
    fn from(_: std::fmt::Error) -> Self {
        TarError::NotGood
    }
}

impl Error for TarError {}

impl UError for TarError {}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let (op, options) = TarOptions::with_operation(&matches)?;

    op.exec(&options)
}

// Commands are grouped to derive "Areas" of tar execution.
// To allow for easier access while parsing and ensuring the
// mutral exclusion of certain arguments like "only one use at a time
// of Acdtrux"
#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        // Since -h flag is used for --dereference for some reason in GNU tar?
        .disable_help_flag(true)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .groups([
            ArgGroup::new("operations").required(true),
            ArgGroup::new("compression"),
            ArgGroup::new("options").multiple(true),
        ])
        .args([
            // Main operation modes
            arg!(-A --catenate "Append tar files to archive").group("operations"),
            arg!(-c --create "Create a new archive").group("operations"),
            arg!(-d --diff "Find differences between archive and file system")
                .alias("compare")
                .group("operations"),
            arg!(-r --append "Append files to end of archive").group("operations"),
            arg!(-t --list "List contents of archive").group("operations"),
            arg!(-u --update "Only append files newer than copy in archive").group("operations"),
            arg!(-x --extract "Extract files from archive")
                .alias("get")
                .group("operations"),
            // Archive file
            Arg::new("archive")
                .long("file")
                .short('f')
                .help("Use archive file")
                .value_parser(clap::value_parser!(PathBuf))
                .group("options"),
            // Compression options
            arg!(-z --gzip "Filter through gzip").group("compression"),
            arg!(-j --bzip2 "Filter through bzip2").group("compression"),
            arg!(-J --xz "Filter through xz").group("compression"),
            // Common options
            arg!(-v --verbose "Verbosely list files processed").group("options"),
            arg!(-h --dereference "Follow symlinks").group("options"),
            // custom long help
            Arg::new("help").long("help").action(ArgAction::Help),
            // arg macro has an issue with the '-' in the middle of the long args
            Arg::new("preserve-permissions")
                .short('p')
                .long("preserve-permissions")
                .action(clap::ArgAction::SetTrue)
                .group("options"),
            Arg::new("absolute-names")
                .short('P')
                .long("absolute-names")
                .action(clap::ArgAction::SetTrue)
                .group("options"),
            // Files to process
            Arg::new("files")
                .help("Files to archive or extract")
                .value_parser(clap::value_parser!(PathBuf))
                .num_args(0..)
                .group("options"),
        ])
}

#[cfg(test)]
mod tests {}
