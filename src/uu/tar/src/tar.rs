// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod archive;
mod options;

use crate::archive::{Archive, Header, Member};
use clap::{arg, crate_version, Arg, ArgAction, Command};
use std::fmt::Formatter;
use std::io::{Read, Seek};
use std::path::PathBuf;
use uucore::error::UResult;
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] [ARG...]";

#[derive(Debug)]
pub enum TarError {
    NotGood,
    InvalidMagic,
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
            Ok(a) => {
               for archive in a {
                   for member in archive.members() {
                       print_member(member);
                   }
               }
            }
            Err(e) => {
                println!("Error: {:#?}", e)
            }
        }
    };

    Ok(())
}

fn extract_archive(tar_file: &PathBuf) -> Result<(), TarError> {
    let options = 2_usize.pow(30);
    let mut archive = read_archive(tar_file)?;
    let file = std::fs::File::open(tar_file).map_err(|e| TarError::NotGood).unwrap();
    let mut reader = std::io::BufReader::new(file);

    let mut block: Vec<u8> = vec![0_u8; options];
    while let Ok(_) = reader.read(block.as_mut_slice()) {
        let current_seek = reader.stream_position().map_err(|_| TarError::NotGood).unwrap();
        // offset since the buffer will be relative index
        let read_start = current_seek.saturating_sub(options as u64);
        for member in archive.members() {
            let size = member.header().size(); 
            let path = member.header();
            let start = member.data_start();
            let end = start + size;
            // extract location of member file
            //let target_location = PathBuf::from();
        }
    }



    Ok(())
}

fn print_member(member: &Member) {
    let header = member.header();
    let mode_str = format_mode(header.mode());
    println!("{} {}/{} {:>11} {} {}",
        mode_str,
        header.uid(),
        header.gid(),
        header.size(),
        header.mtime().strftime("%Y-%m-%d %H:%M"),
        header.name()
    );
}
fn format_mode(mode: u16) -> String {
    let mut buf = ['-'; 10];
    if let None = 1000u16.checked_div(mode).take_if(|x| *x > 0) {
        buf[0] = 'd';
    }
    let owner = mode / 100;
    let group = (mode / 10) % 10;
    let other = mode % 10;
    mode_octal_to_string(owner, &mut buf[1..4]);
    mode_octal_to_string(group, &mut buf[4..7]);
    mode_octal_to_string(other, &mut buf[7..]); 
    String::from_iter(buf) 
}
/// Formats stand linux octal permissions (eg. 0744)
fn mode_octal_to_string(mode: u16, buf: &mut [char]) {
    // example 644
    // | 6 | 4 | 4 |
    //  110 100 100
    //  rw- r-- r--
    buf[0] = if (mode & 0b100) > 0 {'r'}else{'-'};
    buf[1] = if (mode & 0b010) > 0 {'w'}else{'-'};
    buf[2] = if (mode & 0b001) > 0 {'x'}else{'-'};
}

fn read_archive(tar_file: &PathBuf) -> Result<Archive, TarError> {
    // NOTE: this needs many more options to work correctly
    // with all versions of TAR
    let header_size = 512;
    let arch_file = std::fs::File::open(tar_file).unwrap();
    let mut arch_reader = std::io::BufReader::new(arch_file);
    let mut empty_blocks: usize = 0;

    let mut members = vec![];

    let mut block: Vec<u8> = vec![0_u8; header_size];
    while let Ok(_) = arch_reader.read_exact(block.as_mut_slice()) {
        if !block.iter().all(|x| *x == 0) {
            match Header::parse(&block[..header_size]) {
                Ok(header) => {
                    let current_pos = arch_reader.stream_position().unwrap() as usize;
                    let mut seek_size = 512_usize.max(header.size());
                    // check 512 byte boundry
                    if let Some(r) = seek_size.checked_rem(512).take_if(|x| *x > 0) {
                        let pad = 512_usize.saturating_sub(r);
                        seek_size += pad;
                    }
                    members.push(Member::new(
                        header,
                        current_pos.saturating_sub(header_size),
                        current_pos,
                    ));
                    arch_reader.seek_relative(seek_size as i64).unwrap();
                    empty_blocks = 0;
                }
                Err(e) => return Err(e),
            }
        } else {
            // end of archive is 2 empty blocks in a row
            empty_blocks += 1;
            if empty_blocks > 1 {
                break;
            }
        }
    }
    Ok(Archive::new(members))
}

fn read_archives(tar_files: &[&PathBuf]) -> Result<Vec<Archive>, TarError> {
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
