// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod archive;

use clap::{arg, crate_version, Arg, ArgAction, Command};
use jiff::Timestamp;
use std::io::{Read, Seek};
use std::path::PathBuf;
use uucore::error::UResult;
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] [ARG...]";
const TAR_MAGIC: &str = "ustar";

#[derive(Debug)]
enum TarError {
    NotGood,
    InvalidMagic
}

#[derive(Debug)]
enum TarType {
    Normal = 0_isize,
}

impl TryFrom<isize> for TarType {
    type Error = TarError;
    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => TarType::Normal,
            _ => return Err(TarError::NotGood),
        })
    }
}


#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    // For now, just print a basic message indicating the command was parsed
    println!("tar: basic implementation - command line parsed successfully");

    let mut file_names = vec![];

    if matches.get_flag("list") {
        if let Some(file) = matches.get_one::<PathBuf>("file") {
            file_names.push(file);
        }
        if let Some(files) = matches.get_many::<PathBuf>("files") {
            for file in files {
                file_names.push(file);
            }
        }
        match read_archives(&file_names) {
            Ok(h) => { println!("headers: {:#?}", h)},
            Err(e) => {println!("Error: {:#?}", e)}
        }
    };

    Ok(())
}

fn read_archive(tar_file: &PathBuf) -> Result<TarArchive, TarError> {

    // NOTE: this needs many more options to work correctly
    // with all versions of TAR
    let header_size = 512;
    let arch_file = std::fs::File::open(tar_file).unwrap();
    let mut arch_reader = std::io::BufReader::new(arch_file);
    let mut empty_blocks: usize = 0;

    let mut members = vec![];
    
    let mut block: Vec<u8> = vec![0_u8; header_size]; 
    while let Ok(_) = arch_reader.read_exact(block.as_mut_slice()) {
        if !block.iter().all(|x| *x == 0){
            match TarHeader::parse(&block[..header_size]) {
                Ok(header) => {
                    let current_pos = arch_reader.stream_position().unwrap() as usize;
                    let seek_size = 512_i64.max(header.size as i64);
                    members.push(TarArchiveMember {
                        header,
                        header_start: current_pos.saturating_sub(header_size),
                        data_start: current_pos,
                    });
                    arch_reader.seek_relative(seek_size).unwrap();
                    empty_blocks = 0;
                },
                Err(e) => return Err(e)
            }         
        } else {
            // end of archive is 2 empty blocks in a row
            empty_blocks += 1;
            if empty_blocks > 1 {
                break;
            }
        }
    }
    Ok(TarArchive{ members })
}

fn read_archives(tar_files: &[&PathBuf]) -> Result<Vec<TarArchive>, TarError> {

    let mut archives = vec![];

    for file_name in tar_files {
        archives.push(read_archive(file_name)?);
    }
    Ok(archives)
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
