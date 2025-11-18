use crate::errors::TarError;
use crate::operations::operation::TarOperation;
use crate::options::{TarOption, TarParams};
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use std::fmt::Write;
use std::fs::File;
use tar::Archive;
use uucore::error::{UResult, USimpleError};

pub(crate) struct List;

impl TarOperation for List {
    fn exec(&self, options: &TarParams) -> UResult<()> {
        // TODO: I think there is some sort of option to list a
        // particular member
        let mut archive = Archive::new(File::open(options.archive())?);
        let verbose = options
            .options()
            .iter()
            .any(|x| matches!(x, TarOption::Verbose));
        for entry in archive
            .entries()
            .map_err(|x| TarError::TarOperationError(x.to_string()))?
        {
            let e = entry.map_err(|x| TarError::TarOperationError(x.to_string()))?;
            print_entry(e, verbose)?;
        }
        Ok(())
    }
}

fn print_entry(entry: tar::Entry<File>, verbose: bool) -> UResult<()> {
    let header = entry.header();
    let mut line_to_print = String::new();

    if verbose {
        let perm_str = format_perms(header.mode()?);
        // select to use the username/groupname string or uid/gid
        let (u_val, g_val) =
            if let (Ok(Some(un)), Ok(Some(gn))) = (header.username(), header.groupname()) {
                if !un.is_empty() && !gn.is_empty() {
                    (un.to_owned(), gn.to_owned())
                } else {
                    (
                        header.uid()?.to_string(),
                        header.gid()?.to_string(),
                    )
                }
            } else {
                (
                    header.uid()?.to_string(),
                    header.gid()?.to_string(),
                )
            };
        // UNIX tar has this as the minimum size of the Username/id Groupname/id + size
        // section of a listing the anything under 19 is padded over 19 grows and gets
        // padded with 1 space
        // Something in me feels like this could overflow stdout some how?
        let ugs_size: usize = 19;
        let pad = ugs_size.saturating_sub(
            u_val.len() + 1 + g_val.len() + 1 + header.size()?.to_string().len(),
        );
        let mut pad_string = String::new();
        // pad with spaces
        for _ in 0..=pad {
            pad_string.push(' ');
        }
        // construct the combo string with padding
        let ugs = format!(
            "{}/{}{}{}",
            u_val,
            g_val,
            pad_string,
            header.size()?
        );

        // Wrap to jiff timestamps
        let mtime_zoned = Zoned::new(
            // TODO: More descriptive errors needed
            Timestamp::new(header.mtime()?.try_into().expect("Couldnt convert mtime"
            ), 0).map_err(|_| USimpleError::new(1, "Invalid mtime timestamp"))?,
            TimeZone::system(),
        );

        write!(
            &mut line_to_print,
            "{} {} {} {}",
            perm_str,
            ugs,
            mtime_zoned.strftime("%Y-%m-%d %H:%M"),
            header.path().unwrap().display()
        )
        .map_err(|x| TarError::TarOperationError(x.to_string()))?;
    } else {
        write!(&mut line_to_print, "{}", header.path()?.display())
            .map_err(|x| TarError::TarOperationError(x.to_string()))?;
    }
    // print string buffer
    println!("{}", line_to_print);
    Ok(())
}

pub fn format_perms(mode: u32) -> String {
    let mut buf = ['-'; 10];
    // check for the directory flag and set to 'd' if present
    if 1000u32.checked_div(mode).take_if(|x| *x > 0).is_none() {
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
/// Writes to a supplied buffer the char representation of a
/// single standard linux octal permission (eg. 0..|7|..44)
// feeding the mutable buffer in allows default characters to
// added by the caller
pub fn mode_octal_to_string(mode: u32, buf: &mut [char]) {
    // example 644
    // | 6 | 4 | 4 |
    //  110 100 100
    //  rw- r-- r--
    buf[0] = if (mode & 0b100) > 0 { 'r' } else { buf[0] };
    buf[1] = if (mode & 0b010) > 0 { 'w' } else { buf[1] };
    buf[2] = if (mode & 0b001) > 0 { 'x' } else { buf[2] };
}
