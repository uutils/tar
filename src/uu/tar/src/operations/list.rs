// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::TarError;
use crate::CompressionMode;
use chrono::{TimeZone, Utc};
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;
use uucore::fs::display_permissions_unix;

/// List the contents of a tar archive, printing one entry per line.
pub fn list_archive(
    input: impl Read,
    _archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(input, compression)?;
    let mut archive = Archive::new(reader);
    let mut out = BufWriter::new(io::stdout().lock());
    let c_locale = member_name_c_locale();

    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let entry = entry_result.map_err(TarError::CannotReadEntry)?;

        if verbose {
            write_verbose_entry(&entry, &mut out, c_locale)?;
        } else {
            let path = entry.path().map_err(TarError::CannotReadEntryPath)?;
            write_member_name(&mut out, &path, c_locale).map_err(TarError::Io)?;
            writeln!(out).map_err(TarError::Io)?;
        }
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}

fn member_name_c_locale() -> bool {
    // Windows keeps ordinary non-ASCII names as text regardless of locale variables.
    if cfg!(windows) {
        return false;
    }
    member_name_c_locale_for(
        std::env::var_os("LC_ALL").as_deref(),
        std::env::var_os("LC_CTYPE").as_deref(),
        std::env::var_os("LANG").as_deref(),
    )
}

fn member_name_c_locale_for(
    all: Option<&std::ffi::OsStr>,
    ctype: Option<&std::ffi::OsStr>,
    lang: Option<&std::ffi::OsStr>,
) -> bool {
    let locale = [all, ctype, lang]
        .into_iter()
        .flatten()
        .find(|value| !value.is_empty());
    locale.is_none_or(|value| value == "C" || value == "POSIX")
}

fn plain_character(ch: char) -> bool {
    // Line and paragraph separators can split a displayed member name.
    !ch.is_control() && !matches!(ch, '\\' | '\u{2028}' | '\u{2029}')
}

fn write_member_name(out: &mut impl Write, path: &Path, c_locale: bool) -> io::Result<()> {
    let bytes = path.as_os_str().as_encoded_bytes();
    if bytes
        .iter()
        .all(|&byte| byte.is_ascii() && plain_character(byte as char))
        || (!c_locale
            && std::str::from_utf8(bytes).is_ok_and(|name| name.chars().all(plain_character)))
    {
        return out.write_all(bytes);
    }
    if c_locale {
        for &byte in bytes {
            write_quoted_byte(out, byte)?;
        }
        return Ok(());
    }
    for chunk in bytes.utf8_chunks() {
        write_valid_text(out, chunk.valid())?;
        for byte in chunk.invalid() {
            write_octal(out, *byte)?;
        }
    }
    Ok(())
}

fn write_valid_text(out: &mut impl Write, valid: &str) -> io::Result<()> {
    for ch in valid.chars() {
        if ch.is_ascii() {
            write_quoted_byte(out, ch as u8)?;
        } else {
            let mut buffer = [0; 4];
            let bytes = ch.encode_utf8(&mut buffer).as_bytes();
            if !plain_character(ch) {
                for &byte in bytes {
                    write_octal(out, byte)?;
                }
            } else {
                out.write_all(bytes)?;
            }
        }
    }
    Ok(())
}

fn write_quoted_byte(out: &mut impl Write, byte: u8) -> io::Result<()> {
    if byte.is_ascii() && plain_character(byte as char) {
        return out.write_all(&[byte]);
    }
    match byte {
        b'\\' => out.write_all(b"\\\\"),
        7 => out.write_all(b"\\a"),
        8 => out.write_all(b"\\b"),
        b'\t' => out.write_all(b"\\t"),
        b'\n' => out.write_all(b"\\n"),
        11 => out.write_all(b"\\v"),
        12 => out.write_all(b"\\f"),
        b'\r' => out.write_all(b"\\r"),
        _ => write_octal(out, byte),
    }
}

fn write_octal(out: &mut impl Write, byte: u8) -> io::Result<()> {
    write!(out, "\\{byte:03o}")
}

fn write_verbose_entry<R: Read>(
    entry: &tar::Entry<'_, R>,
    out: &mut impl Write,
    c_locale: bool,
) -> Result<(), TarError> {
    let (mode, entry_type, owner, group, size, mtime) = {
        let header = entry.header();
        (
            header.mode().unwrap_or(0),
            header.entry_type(),
            header
                .username()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_owned(),
            header
                .groupname()
                .ok()
                .flatten()
                .unwrap_or_default()
                .to_owned(),
            header.size().unwrap_or(0),
            header.mtime().unwrap_or(0),
        )
    };

    let path = entry.path().map_err(TarError::CannotReadEntryPath)?;

    let type_char = match entry_type {
        tar::EntryType::Directory => 'd',
        tar::EntryType::Symlink => 'l',
        tar::EntryType::Char => 'c',
        tar::EntryType::Block => 'b',
        tar::EntryType::Fifo => 'p',
        _ => '-',
    };
    let perm_str = display_permissions_unix(mode, false);
    let permissions = format!("{type_char}{perm_str}");

    let dt: chrono::DateTime<Utc> = Utc
        .timestamp_opt(mtime as i64, 0)
        .single()
        .unwrap_or_else(Utc::now);
    let date_str = dt.format("%Y-%m-%d %H:%M");

    write!(out, "{permissions} {owner}/{group} {size:>8} {date_str} ").map_err(TarError::Io)?;
    write_member_name(out, &path, c_locale).map_err(TarError::Io)?;
    writeln!(out).map_err(TarError::Io)
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
