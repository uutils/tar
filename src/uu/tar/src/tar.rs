// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod archive;
mod options;
mod operation;
mod list;
mod util;

use crate::archive::{Archive, Header, Member};
use crate::options::*;
use crate::operation::{Operation, TarOperation};
use crate::list::ListOperation;
use clap::{arg, crate_version, Arg, ArgAction, Command};
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::error::Error;
use std::fmt::{Display, Debug};
use uucore::error::{UError, UResult};
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] [ARG...]";

pub enum TarError {
    NotGood,
    InvalidMagic,
}

impl Display for TarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TarError::NotGood => {
                f.write_str("TarError: Not Good...This will be replaced later") 
            },
            TarError::InvalidMagic => {
                f.write_str("TarError: Invalid Magic") 
            }
        }
    }
} 
impl Debug for TarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TarError::NotGood => {
                f.write_str("TarError: Not Good...This will be replaced later") 
            },
            TarError::InvalidMagic => {
                f.write_str("TarError: Invalid Magic") 
            }
        }
    }
} 

impl Error for TarError {} 

impl UError for TarError {}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {

    // For now, just print a basic message indicating the command was parsed
    println!("tar: basic implementation - command line parsed successfully");

    let matches = uu_app().try_get_matches_from(args)?;

    let (op, options) = TarOptions::with_operation(&matches);

    op.exec(&options)

}


#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        // Since -h flag is used for --dereference for some reason in GNU tar?
        .disable_help_flag(true)
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
            // custom long help
            Arg::new("help").long("help").action(ArgAction::Help),
            // arg macro has an issue with the '-' in the middle of the long args
            Arg::new("preserve-permissions")
                .short('p')
                .long("preserve-permissions")
                .action(clap::ArgAction::SetTrue),
            Arg::new("absolute-names")
                .short('P')
                .long("absolute-names")
                .action(clap::ArgAction::SetTrue),
            // Files to process
            Arg::new("files")
                .help("Files to archive or extract")
                .value_parser(clap::value_parser!(PathBuf))
                .num_args(0..),
        ])
}

#[cfg(test)]
mod tests {}
