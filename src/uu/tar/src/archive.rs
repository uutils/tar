
use jiff::Timestamp;
use crate::TarType;
use crate::TarError;
use crate::TAR_MAGIC;


/// Contains 3 values the represent the length in bytes
/// The offset into the header according to its spec
/// and the optional place for the value while parsing 
/// from the builder
#[derive(Debug)]
struct TarMeta<T: Sized + std::fmt::Debug> {
    len: usize,
    offset: usize,
    value: Option<T>
}



// NOTE: equivelent data sizes used rust crate libc maintains typedef alias's
// that define things like uid_t or gid_t might move these to libc since this
// is self contained don't even know if it is necessary
//
/// Builder Struct to parse header data to hopefully make
/// a best guess choice at possible header of archive member
/// Combined Header Metadata based on definitions
/// from v7Unix/POSIX 1003.1-1990/Star/GNU/UStar
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

#[derive(Debug)]
pub struct TarHeader {
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
impl Default for TarHeader {
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
impl TarHeader {
    pub fn parse(block: &[u8]) -> Result<Self, TarError> {
        TarHeaderBuilder::parse(block)
    }
    pub fn size(&self) -> usize {
        self.size
    }
}
impl Default for TarHeaderBuilder {
    fn default() -> TarHeaderBuilder {
        TarHeaderBuilder {
            name: TarMeta::<String> {len: 100, offset: 0, value: None},
            mode: TarMeta::<u16> {len: 8, offset: 100, value: None},
            uid: TarMeta::<u32> {len: 8, offset: 108, value: None},
            gid: TarMeta::<u32> {len: 8, offset: 116, value: None},
            size: TarMeta::<usize> {len: 12, offset: 124, value: None},
            mtime: TarMeta::<Timestamp> {len: 12, offset: 136, value: None},
            chksum: TarMeta::<usize> {len: 8, offset: 148, value: None},
            typeflag: TarMeta::<TarType> {len: 1, offset: 156, value: None},
            linkname: TarMeta::<String> {len: 100, offset: 157, value: None},
            magic: TarMeta::<String> {len: 6, offset: 257, value: None},
            version: TarMeta::<u16> {len: 2, offset: 263, value: None},
            uname: TarMeta::<String> {len: 32, offset: 265, value: None},
            gname: TarMeta::<String> {len: 32, offset: 297, value: None},
            devmajor: TarMeta::<usize> {len: 8, offset: 329, value: None},
            devminor: TarMeta::<usize> {len: 8, offset: 337, value: None},
            prefix: TarMeta::<String> {len: 155, offset: 345, value: None},
            star_prefix: TarMeta::<String> {len: 131, offset: 345, value: None},
            atime: TarMeta::<Timestamp> {len: 12, offset: 476, value: None},
            ctime: TarMeta::<Timestamp> {len: 12, offset: 488, value: None},
        }
    }
}
impl TarHeaderBuilder {
    fn new() -> Self {
        TarHeaderBuilder {
            ..Default::default()
        }
    }
    pub fn parse(block: &[u8]) -> Result<TarHeader, TarError> {
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
        /*
        Ok(Self {
            magic: {
                let magic_str = block[offsets.magic..offsets.version]
                .iter()
                .map(|x| *x as char)
                .filter(|x| x.is_ascii() && *x != ' ')
                .collect::<String>();

                if magic_str != TAR_MAGIC {
                    //return Err(TarError::InvalidMagic)
                }

                magic_str
            },
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
        */
        println!("builder: {:?}", builder);
        Ok(TarHeader {
            name: builder.name.value.unwrap(),
            mode: builder.mode.value.unwrap(),
            uid: builder.uid.value.unwrap(),
            gid: builder.gid.value.unwrap(),
            size: builder.size.value.unwrap(),
            mtime: builder.mtime.value.unwrap(),
            chksum: builder.chksum.value.unwrap(),
            typeflag: builder.typeflag.value.unwrap(),
            linkname: builder.linkname.value.unwrap(),
            ..Default::default()   
        })
    }
}

#[derive(Debug)]
pub struct TarArchive {
    members: Vec<TarArchiveMember>
}
impl TarArchive {
    pub fn new(members: Vec<TarArchiveMember>) -> Self {
        Self { members }
    }
}
#[derive(Debug)]
pub struct TarArchiveMember {
    header: TarHeader,
    header_start: usize,
    data_start: usize
}
impl TarArchiveMember {
    pub fn new(header: TarHeader, header_start: usize, data_start: usize) -> Self {
        Self {
            header,
            header_start,
            data_start
        }
    }
}

// TarArchive
// |
// |-Header
//  |-Metadata
//      |-Member

trait TarParse<'i> {
    type Input;
    fn parse_field(&mut self, val: Self::Input);    
}

impl<'i> TarParse<'i> for TarMeta<String> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(val[self.offset..self.offset+self.len].iter()
                .filter(|x| **x != 0 && x.is_ascii())
                .map(|c| *c as char)
                .collect::<String>())
    }
}

impl<'i> TarParse<'i> for TarMeta<u16> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(val[self.offset..self.offset+self.len]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0))
    }
}

impl<'i> TarParse<'i> for TarMeta<u32> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(val[self.offset..self.offset+self.len]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0))
    }
}

impl<'i> TarParse<'i> for TarMeta<usize> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(usize::from_str_radix(
                &val[self.offset..self.offset+self.len].iter()
                .filter(|x| **x != 0 && x.is_ascii())
                .map(|c| *c as char)
                .collect::<String>()
                , 8).unwrap_or(0),)
    }
}

impl<'i> TarParse<'i> for TarMeta<TarType> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        self.value = Some(TarType::try_from(
                val[self.offset..self.offset+self.len]
                .iter()
                .filter(|x| x.is_ascii_digit())
                .map(|x| *x as char)
                .collect::<String>()
                .parse()
                .unwrap_or(0),
            )
            .unwrap_or(TarType::Normal))
    }
}

impl<'i> TarParse<'i> for TarMeta<Timestamp> {
    type Input = &'i [u8];
    fn parse_field(&mut self, val: Self::Input) {
        println!("val: {:#?}", &val[self.offset..self.offset+self.len]);
        self.value = Some(Timestamp::from_second(
                i64::from_str_radix({
                    &val[self.offset..self.offset+self.len]
                    .iter()
                    .filter(|x| x.is_ascii_digit())
                    .map(|x| *x as char)
                    .collect::<String>()
                }, 8)
                .unwrap_or(0)
            ).unwrap()
        )
    }
}
