use crate::{TAR_MAGIC, TarType, TarError};
use jiff::Timestamp;

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
pub struct TarHeader {
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
            magic: {
                let magic_str = block[offsets.magic..offsets.version]
                .iter()
                .map(|x| *x as char)
                .filter(|x| x.is_ascii() && *x != ' ')
                .collect::<String>();

                if magic_str != TAR_MAGIC {
                    return Err(TarError::InvalidMagic)
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
    }
}

#[derive(Debug)]
pub struct TarArchive {
    members: Vec<TarArchiveMember>
}
#[derive(Debug)]
pub struct TarArchiveMember {
    header: TarHeader,
    header_start: usize,
    data_start: usize
}

// TarArchive
// |
// |-Header
//  |-Metadata
//      |-Member

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
