// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::TarError;
use crate::CompressionMode;
use chrono::{Local, TimeZone};
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
    // Width of the "owner/group size" column; it grows for a wider entry and
    // stays wide for the rest of the listing, as in GNU tar.
    let mut owner_size_width = 19;

    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let entry = entry_result.map_err(TarError::CannotReadEntry)?;

        if verbose {
            let formatted = format_verbose_entry(&entry, &mut owner_size_width)?;
            writeln!(out, "{formatted}").map_err(TarError::Io)?;
        } else {
            let path = entry.path().map_err(TarError::CannotReadEntryPath)?;
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}

fn format_verbose_entry<R: Read>(
    entry: &tar::Entry<'_, R>,
    owner_size_width: &mut usize,
) -> Result<String, TarError> {
    let (mode, entry_type, owner, group, size, mtime) = {
        let header = entry.header();
        // Without a user or group name GNU tar shows the numeric id.
        let name_or_id = |name: Option<&str>, id: Option<u64>| match name {
            Some(name) if !name.is_empty() => name.to_owned(),
            _ => id.unwrap_or(0).to_string(),
        };
        (
            header.mode().unwrap_or(0),
            header.entry_type(),
            name_or_id(header.username().ok().flatten(), header.uid().ok()),
            name_or_id(header.groupname().ok().flatten(), header.gid().ok()),
            header.size().unwrap_or(0),
            header.mtime().unwrap_or(0),
        )
    };

    let path = entry.path().map_err(TarError::CannotReadEntryPath)?;

    let type_char = match entry_type {
        tar::EntryType::Directory => 'd',
        tar::EntryType::Symlink => 'l',
        tar::EntryType::Link => 'h',
        tar::EntryType::Char => 'c',
        tar::EntryType::Block => 'b',
        tar::EntryType::Fifo => 'p',
        _ => '-',
    };
    let perm_str = display_permissions_unix(mode, false);
    let permissions = format!("{type_char}{perm_str}");

    // GNU tar shows times in the local time zone
    let dt: chrono::DateTime<Local> = Local
        .timestamp_opt(mtime as i64, 0)
        .single()
        .unwrap_or_else(Local::now);
    let date_str = dt.format("%Y-%m-%d %H:%M");

    let link_str = match (entry_type, entry.link_name().ok().flatten()) {
        (tar::EntryType::Symlink, Some(target)) => format!(" -> {}", target.display()),
        (tar::EntryType::Link, Some(target)) => format!(" link to {}", target.display()),
        _ => String::new(),
    };

    let owner_group = format!("{owner}/{group}");
    let size = size.to_string();
    *owner_size_width = (*owner_size_width).max(owner_group.len() + 1 + size.len());
    let size_width = *owner_size_width - owner_group.len() - 1;
    Ok(format!(
        "{permissions} {owner_group} {size:>size_width$} {date_str} {}{link_str}",
        path.display()
    ))
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
