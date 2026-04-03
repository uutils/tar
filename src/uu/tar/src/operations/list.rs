// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use chrono::{TimeZone, Utc};
use std::fs::File;
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;
use uucore::fs::display_permissions_unix;

/// List the contents of a tar archive, printing one entry per line.
pub fn list_archive(archive_path: &Path, verbose: bool) -> UResult<()> {
    let file: File =
        File::open(archive_path).map_err(|e| TarError::from_io_error(e, archive_path))?;
    let mut archive = Archive::new(file);

    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let entry = entry_result.map_err(TarError::CannotReadEntry)?;

        if verbose {
            // Collect all header fields into owned values before borrowing entry for the path,
            // since both header() and path() require a borrow of entry.
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
            // Tar headers store the type separately from the mode bits, so we get the
            // 9-character rwx string from uucore and prepend our own type character.
            let perm_str = display_permissions_unix(mode, false);
            let permissions = format!("{type_char}{perm_str}");

            // TODO: GNU tar displays mtime in the user's local timezone; we
            // currently format in UTC. Convert to local time for compatibility.
            let dt: chrono::DateTime<Utc> = Utc
                .timestamp_opt(mtime as i64, 0)
                .single()
                .unwrap_or_else(Utc::now);
            let date_str = dt.format("%Y-%m-%d %H:%M");

            println!(
                "{permissions} {owner}/{group} {size:>8} {date_str} {}",
                path.display()
            );
        } else {
            let path = entry.path().map_err(TarError::CannotReadEntryPath)?;

            println!("{}", path.display());
        }
    }

    Ok(())
}
