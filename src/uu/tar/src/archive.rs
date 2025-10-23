use crate::operation::TarOperation;
use crate::options::TarOptions;
use crate::util::*;
use crate::TarError;
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use std::io::{Read, Seek};
use std::path::PathBuf;
use uucore::error::{UError, UResult};

#[allow(dead_code)]
const USTAR_MAGIC: &'static str = "ustar ";

// create operation new type
// NOTE: Might move to a new file or not I am not sure
pub(crate) struct CreateOperation;

/// [`CreateOperation`] is not impl'd
#[allow(unused_variables)]
impl TarOperation for CreateOperation {
    fn exec(&self, options: &TarOptions) -> UResult<()> {
        todo!()
    }
}

/// New type to leverage from trait to produce a
/// Vector of Archives while reading
pub struct ArchiveList(Vec<Archive>);

impl Into<Vec<Archive>> for ArchiveList {
    fn into(self) -> Vec<Archive> {
        self.0
    }
}

impl IntoIterator for ArchiveList {
    type Item = Archive;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// [`TarType`] Enum to store selected value for the typeflag
/// field within a [`Header`] in addition to its int value for
/// searlization and deserialization
#[derive(Debug)]
#[allow(dead_code)]
pub enum TarType {
    Normal = 0_isize,
    HardLink = 1_isize,
    SymbolicLink = 2_isize,
    CharacterSpecial = 3_isize,
    BlockSpecial = 4_isize,
    Directory = 5_isize,
    FIFO = 6_isize,
    Contiguous = 7_isize,
    ExtendedHeader = b'g' as isize,
    ExtendedNext = b'x' as isize,
}

impl TryFrom<isize> for TarType {
    type Error = TarError;
    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => TarType::Normal,
            1 => TarType::HardLink,
            2 => TarType::SymbolicLink,
            3 => TarType::CharacterSpecial,
            4 => TarType::BlockSpecial,
            5 => TarType::Directory,
            6 => TarType::FIFO,
            7 => TarType::Contiguous,
            _ => return Err(TarError::NotGood),
        })
    }
}

/// [`TarMeta`] describes [`Header`] fields and holds its
/// resulting parsed value to be later converted to rust types.
/// Allowing for the creation of a [`TarHeaderBuilder`] and subsequent
/// [`TarParse`] impls to be used based on the data type choosen for conversion
#[derive(Debug)]
struct TarMeta<T: Sized + std::fmt::Debug> {
    len: usize,
    offset: usize,
    value: Option<T>,
}

/// [`TarHeaderBuilder`] captures and stores values as [`TarMeta`]
/// data for later conversion to a [`Header`] struct
/// Combined Header Metadata based on definitions
/// from v7Unix/POSIX 1003.1-1990/Star/GNU/UStar
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
/// prefix        345            155/131
/// atime         476            12
/// ctime         488            12
#[derive(Debug)]
pub struct TarHeaderBuilder {
    name: TarMeta<String>,
    mode: TarMeta<u16>,
    uid: TarMeta<u32>,
    gid: TarMeta<u32>,
    size: TarMeta<usize>,
    mtime: TarMeta<Timestamp>,
    chksum: TarMeta<usize>,
    typeflag: TarMeta<TarType>,
    linkname: TarMeta<String>,
    magic: TarMeta<String>,
    version: TarMeta<u16>,
    uname: TarMeta<String>,
    gname: TarMeta<String>,
    devmajor: TarMeta<usize>,
    devminor: TarMeta<usize>,
    prefix: TarMeta<String>,
    star_prefix: TarMeta<String>,
    atime: TarMeta<Timestamp>,
    ctime: TarMeta<Timestamp>,
}

/// [`Header`] stores information of archive memebers as a
/// result of parsing and construction of [`TarHeaderBuilder`]
/// name, mode, uid, gid, size, mtime, chksum, typeflag, and
/// linkname are ubiquidous across all tar standards.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Header {
    name: String,
    mode: u16,
    uid: u32,
    gid: u32,
    size: usize,
    mtime: Timestamp,
    chksum: usize,
    typeflag: TarType,
    linkname: String,
    magic: Option<String>,
    version: Option<u16>,
    uname: Option<String>,
    gname: Option<String>,
    devmajor: Option<usize>,
    devminor: Option<usize>,
    prefix: Option<String>,
    star_prefix: Option<String>,
    atime: Option<Timestamp>,
    ctime: Option<Timestamp>,
}
impl Default for Header {
    fn default() -> Self {
        Self {
            name: String::from(""),
            mode: 0u16,
            uid: 0u32,
            gid: 0u32,
            size: 0usize,
            mtime: Timestamp::default(),
            chksum: 0usize,
            typeflag: TarType::Normal,
            linkname: String::from(""),
            magic: None,
            version: None,
            uname: None,
            gname: None,
            devmajor: None,
            devminor: None,
            prefix: None,
            star_prefix: None,
            atime: None,
            ctime: None,
        }
    }
}
#[allow(dead_code)]
impl Header {
    pub fn parse(block: &[u8]) -> Result<Self, TarError> {
        TarHeaderBuilder::parse(block)
    }
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn mode(&self) -> u16 {
        self.mode
    }
    pub fn uid(&self) -> u32 {
        self.uid
    }
    pub fn gid(&self) -> u32 {
        self.gid
    }
    pub fn mtime(&self) -> &Timestamp {
        &self.mtime
    }
    // convience method to create a locale zoned
    // time stamp for display. C tar does no conversion
    // of time and the tar file format timestamp local unix epoch
    // time with no information included
    pub fn mtime_zoned(&self) -> Zoned {
        Zoned::new(self.mtime, TimeZone::system())
    }
    pub fn chksum(&self) -> usize {
        self.size
    }
}
impl Default for TarHeaderBuilder {
    fn default() -> TarHeaderBuilder {
        TarHeaderBuilder {
            name: TarMeta::<String> {
                len: 100,
                offset: 0,
                value: None,
            },
            mode: TarMeta::<u16> {
                len: 8,
                offset: 100,
                value: None,
            },
            uid: TarMeta::<u32> {
                len: 8,
                offset: 108,
                value: None,
            },
            gid: TarMeta::<u32> {
                len: 8,
                offset: 116,
                value: None,
            },
            size: TarMeta::<usize> {
                len: 12,
                offset: 124,
                value: None,
            },
            mtime: TarMeta::<Timestamp> {
                len: 12,
                offset: 136,
                value: None,
            },
            chksum: TarMeta::<usize> {
                len: 8,
                offset: 148,
                value: None,
            },
            typeflag: TarMeta::<TarType> {
                len: 1,
                offset: 156,
                value: None,
            },
            linkname: TarMeta::<String> {
                len: 100,
                offset: 157,
                value: None,
            },
            magic: TarMeta::<String> {
                len: 6,
                offset: 257,
                value: None,
            },
            version: TarMeta::<u16> {
                len: 2,
                offset: 263,
                value: None,
            },
            uname: TarMeta::<String> {
                len: 32,
                offset: 265,
                value: None,
            },
            gname: TarMeta::<String> {
                len: 32,
                offset: 297,
                value: None,
            },
            devmajor: TarMeta::<usize> {
                len: 8,
                offset: 329,
                value: None,
            },
            devminor: TarMeta::<usize> {
                len: 8,
                offset: 337,
                value: None,
            },
            prefix: TarMeta::<String> {
                len: 155,
                offset: 345,
                value: None,
            },
            star_prefix: TarMeta::<String> {
                len: 131,
                offset: 345,
                value: None,
            },
            atime: TarMeta::<Timestamp> {
                len: 12,
                offset: 476,
                value: None,
            },
            ctime: TarMeta::<Timestamp> {
                len: 12,
                offset: 488,
                value: None,
            },
        }
    }
}
impl TarHeaderBuilder {
    fn new() -> Self {
        TarHeaderBuilder {
            ..Default::default()
        }
    }
    pub fn parse(block: &[u8]) -> Result<Header, TarError> {
        let mut builder = TarHeaderBuilder::new();
        builder.name.parse_field(block);
        builder.mode.parse_field(block);
        builder.uid.parse_field(block);
        builder.gid.parse_field(block);
        builder.size.parse_field(block);
        builder.mtime.parse_field(block);
        builder.chksum.parse_field(block);
        builder.typeflag.parse_field(block);
        builder.linkname.parse_field(block);
        builder.magic.parse_field(block);
        builder.version.parse_field(block);
        builder.uname.parse_field(block);
        builder.gname.parse_field(block);
        builder.devmajor.parse_field(block);
        builder.devminor.parse_field(block);
        builder.prefix.parse_field(block);
        builder.star_prefix.parse_field(block);
        builder.atime.parse_field(block);
        builder.ctime.parse_field(block);

        // TODO: replace nOtGoOd error with something better
        Ok(Header {
            name: builder.name.value.ok_or(TarError::NotGood)?,
            mode: builder.mode.value.ok_or(TarError::NotGood)?,
            uid: builder.uid.value.ok_or(TarError::NotGood)?,
            gid: builder.gid.value.ok_or(TarError::NotGood)?,
            size: builder.size.value.ok_or(TarError::NotGood)?,
            mtime: builder.mtime.value.ok_or(TarError::NotGood)?,
            chksum: builder.chksum.value.ok_or(TarError::NotGood)?,
            typeflag: builder.typeflag.value.ok_or(TarError::NotGood)?,
            linkname: builder.linkname.value.ok_or(TarError::NotGood)?,
            magic: builder.magic.value,
            version: builder.version.value,
            uname: builder.uname.value,
            gname: builder.gname.value,
            devmajor: builder.devmajor.value,
            devminor: builder.devminor.value,
            prefix: builder.prefix.value,
            star_prefix: builder.star_prefix.value,
            atime: builder.atime.value,
            ctime: builder.atime.value,
            ..Default::default()
        })
    }
}

#[derive(Debug)]
pub struct Archive {
    members: Vec<Member>,
}
impl TryFrom<&[PathBuf]> for ArchiveList {
    type Error = Box<dyn UError + 'static>;
    fn try_from(files: &[PathBuf]) -> UResult<ArchiveList> {
        Archive::read_archives(files)
    }
}
impl TryFrom<&PathBuf> for Archive {
    type Error = Box<dyn UError + 'static>;
    fn try_from(file: &PathBuf) -> UResult<Archive> {
        Self::read_archive(file)
    }
}
#[allow(dead_code)]
impl Archive {
    pub fn new(members: Vec<Member>) -> Self {
        Self { members }
    }
    pub fn members(&self) -> &Vec<Member> {
        &self.members
    }
    pub fn members_mut(&mut self) -> &mut Vec<Member> {
        &mut self.members
    }

    fn read_archive(tar_file: &PathBuf) -> UResult<Archive> {
        // NOTE: this needs many more options to work correctly
        // with all versions of TAR
        let header_size = 512;
        let arch_file = std::fs::File::open(tar_file).unwrap();
        let mut arch_reader = std::io::BufReader::new(arch_file);
        let mut empty_blocks: usize = 0;

        let mut members = vec![];

        let mut block: Vec<u8> = vec![0_u8; header_size];
        // TODO: configure to work with variable block sizes
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
                    Err(e) => return Err(Box::new(e)),
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

    fn read_archives(tar_files: &[PathBuf]) -> UResult<ArchiveList> {
        let mut archives = vec![];

        for file_name in tar_files {
            archives.push(Self::read_archive(file_name)?);
        }
        Ok(ArchiveList(archives))
    }
}
/// [`Member`] An Archive Member is the indivdual file listing
/// an a tar archive and the [`Member`] struct encapsulates the
/// associated parsed [`Header`] and byte offsets for reading and
/// writing data into an archive.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Member {
    header: Header,
    header_start: usize,
    data_start: usize,
}
#[allow(dead_code)]
impl Member {
    pub fn new(header: Header, header_start: usize, data_start: usize) -> Self {
        Self {
            header,
            header_start,
            data_start,
        }
    }
    pub fn header(&self) -> &Header {
        &self.header
    }
    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }
    pub fn header_start(&self) -> usize {
        self.header_start
    }
    pub fn data_start(&self) -> usize {
        self.data_start
    }
    pub fn print_member(&self, verbose: bool) {
        let header = self.header();
        let mode_str = format_perms(header.mode());
        if verbose {
            println!(
                "{} {}/{} {:>11} {} {}",
                mode_str,
                header.uid(),
                header.gid(),
                header.size(),
                header.mtime_zoned().strftime("%Y-%m-%d %H:%M"),
                header.name()
            );
        } else {
            println!("{}", header.name());
        }
    }
}
// TODO: Must convert errors to actual UUCORE URESULTS and
// also not force the `Some()` return 100% which defeats the purpose
// of having the non-required fields in optionals.
/// [`TarParse`] allows per input type parsing to be defined based on
/// [`Header`] output data type being requested.
trait TarParse<'i> {
    type Input;
    fn parse_field(&mut self, val: Self::Input);
}

impl<'i> TarParse<'i> for TarMeta<String> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(
            val[self.offset..self.offset + self.len]
                .iter()
                .filter(|x| **x != 0 && x.is_ascii())
                .map(|c| *c as char)
                .collect::<String>(),
        )
    }
}

impl<'i> TarParse<'i> for TarMeta<u16> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(
            val[self.offset..self.offset + self.len]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
        )
    }
}

impl<'i> TarParse<'i> for TarMeta<u32> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(
            u32::from_str_radix(
                &val[self.offset..self.offset + self.len]
                    .iter()
                    .filter(|x| x.is_ascii_digit())
                    .map(|x| *x as char)
                    .collect::<String>(),
                8,
            )
            .unwrap_or(0),
        )
    }
}

impl<'i> TarParse<'i> for TarMeta<usize> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(
            usize::from_str_radix(
                &val[self.offset..self.offset + self.len]
                    .iter()
                    .filter(|x| x.is_ascii_digit())
                    .map(|c| *c as char)
                    .collect::<String>(),
                8,
            )
            .unwrap_or(0),
        )
    }
}

impl<'i> TarParse<'i> for TarMeta<TarType> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(
            TarType::try_from(
                val[self.offset..self.offset + self.len]
                    .iter()
                    .filter(|x| x.is_ascii_digit())
                    .map(|x| *x as char)
                    .collect::<String>()
                    .parse::<isize>()
                    .unwrap_or(0),
            )
            .unwrap_or(TarType::Normal),
        )
    }
}

impl<'i> TarParse<'i> for TarMeta<Timestamp> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(
            Timestamp::from_second(
                i64::from_str_radix(
                    {
                        &val[self.offset..self.offset + self.len]
                            .iter()
                            .filter(|x| x.is_ascii_digit())
                            .map(|x| *x as char)
                            .collect::<String>()
                    },
                    8,
                )
                .unwrap_or(0),
            )
            .unwrap(),
        )
    }
}
