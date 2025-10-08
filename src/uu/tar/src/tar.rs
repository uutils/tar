// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{arg, crate_version, Arg, ArgAction, Command};
use jiff::Timestamp;
use std::io::Read;
use std::path::PathBuf;
use uucore::error::UResult;
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] [ARG...]";

#[derive(Debug)]
enum TarError {
    NotGood,
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

// NOTE: equivelent data sizes used rust crate libc maintains typedef alias's
// that define things like uid_t or gid_t might move these to libc since this
// is self contained don't even know if it is necessary

/// Header based on definition POSIX 1003.1-1990
/// Adopted to in memory rust builtin types
/// FIELD NAME    BYTE OFFSET    LENGTH (in bytes)
/// name          0              100
/// mode          100            8
/// uid           108            8
/// gid           116            8
/// size          124            12
/// mtime         136            12
/// chksum        148            8
/// typeflag      156            1
/// linkname      157            100
/// magic         257            6
/// version       263            2
/// uname         265            32
/// gname         297            32
/// devmajor      329            8
/// devminor      337            8
/// prefix        345            155
#[derive(Debug)]
struct TarHeader {
    name: String,
    mode: u16,
    uid: u32,
    gid: u32,
    size: u64,
    mtime: Timestamp,
    chksum: u64,
    typeflag: TarType,
    linkname: String,
    magic: String,
    version: u16,
    uname: String,
    gname: String,
    devmajor: u64,
    devminor: u64,
    prefix: String,
}
impl TarHeader {
    fn parse(block: &[u8]) -> Result<Self, TarError> {
        let offsets = TarHeaderOffsets::default();
        Ok(Self {
            name: block[offsets.name..offsets.mode]
                .iter()
                .filter(|x| **x != 0 && x.is_ascii())
                .map(|c| *c as char)
                .collect::<String>(),
            mode: block[offsets.mode..offsets.uid]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            uid: block[offsets.uid..offsets.gid]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            gid: block[offsets.gid..offsets.size]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            size: block[offsets.size..offsets.mtime]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            // WARN: I dont know why yet this works with base 8
            mtime: Timestamp::from_second(
                i64::from_str_radix({
                    &block[offsets.mtime..offsets.chksum]
                    .iter()
                    .filter(|x| x.is_ascii_digit())
                    .map(|x| *x as char)
                    .collect::<String>()
                }, 8)
                .unwrap_or(0)
            )
            .unwrap(),
            chksum: block[offsets.chksum..offsets.typeflag]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            typeflag: TarType::try_from(
                block[offsets.typeflag..offsets.linkname]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            )
            .unwrap_or(TarType::Normal),
            linkname: block[offsets.linkname..offsets.magic]
                .iter()
                .map(|x| *x as char)
                .filter(|x| x.is_ascii() && *x != '\0')
                .collect::<String>(),
            // NOTE TAR spec includes a null byte at the end of the magic
            // IVR 
            magic: block[offsets.magic..offsets.version]
                .iter()
                .map(|x| *x as char)
                .filter(|x| x.is_ascii() && *x != ' ')
                .collect::<String>(),
            version: block[offsets.version..offsets.uname]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            uname: block[offsets.uname..offsets.gname]
                .iter()
                .filter(|x| **x != 0 && x.is_ascii())
                .map(|c| *c as char)
                .collect::<String>(),
            gname: block[offsets.gname..offsets.devmajor]
                .iter()
                .filter(|x| **x != 0 && x.is_ascii())
                .map(|c| *c as char)
                .collect::<String>(),
            devmajor: block[offsets.devmajor..offsets.devminor]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            devminor: block[offsets.devminor..offsets.prefix]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            prefix: block[offsets.prefix..offsets.end]
                .iter()
                .filter(|x| **x != 0 && x.is_ascii())
                .map(|c| *c as char)
                .collect::<String>(),
        })
    }
}
/// Header based on definition POSIX 1003.1-1990
/// Adopted to in memory rust builtin types
/// FIELD NAME    BYTE OFFSET    LENGTH (in bytes)
/// name          0              100
/// mode          100            8
/// uid           108            8
/// gid           116            8
/// size          124            12
/// mtime         136            12
/// chksum        148            8
/// typeflag      156            1
/// linkname      157            100
/// magic         257            6
/// version       263            2
/// uname         265            32
/// gname         297            32
/// devmajor      329            8
/// devminor      337            8
/// prefix        345            155
struct TarHeaderOffsets {
    name: usize,
    mode: usize,
    uid: usize,
    gid: usize,
    size: usize,
    mtime: usize,
    chksum: usize,
    typeflag: usize,
    linkname: usize,
    magic: usize,
    version: usize,
    uname: usize,
    gname: usize,
    devmajor: usize,
    devminor: usize,
    prefix: usize,
    end: usize,
}
impl Default for TarHeaderOffsets {
    fn default() -> TarHeaderOffsets {
        Self {
            name: 0,
            mode: 100,
            uid: 108,
            gid: 116,
            size: 124,
            mtime: 136,
            chksum: 148,
            typeflag: 156,
            linkname: 157,
            magic: 257,
            version: 263,
            uname: 265,
            gname: 297,
            devmajor: 329,
            devminor: 337,
            prefix: 345,
            end: 500,
        }
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
        //FIXME: Dont panic return URESULT
        read_headers(&file_names).unwrap();
    };

    Ok(())
}

// TODO going to want to parse entirity of all archives and return in mem
// rep of headers + data_ptrs for potential extraction
fn read_headers(tar_files: &[&PathBuf]) -> Result<Vec<TarHeader>, TarError> {
    let mut headers = vec![];

    for file_name in tar_files {
        let mut archive = std::fs::File::open(file_name).unwrap();
        let mut file_bytes = vec![];
        
        // So as this is reading we have to keep in mind the size
        // of the archive since this could potentially be huge
        archive.read_to_end(&mut file_bytes).unwrap();

        let mut chunks = file_bytes.chunks(512);
        while let Some(buf) = chunks.next() {
            match TarHeader::parse(buf) {
                Ok(head) => {
                    if head.size > 0 {
                        let data_size: usize = if head.size < 513 { 1 } else { head.size as usize / 512 };
                        for _ in 0..data_size {
                            chunks.next();
                        }
                        headers.push(head);
                    }
                },
                Err(e) => return Err(e)
            }         
        }
    }
    Ok(headers)
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
