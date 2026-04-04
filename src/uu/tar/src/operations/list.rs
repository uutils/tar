// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::compression::open_archive_reader;
use crate::CompressionMode;
use chrono::{TimeZone, Utc};
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;
use uucore::fs::display_permissions_unix;

/// List the contents of a tar archive, printing one entry per line.
pub fn list_archive(
    archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(archive_path, compression)?;
    let mut archive = Archive::new(reader);

    for entry_result in archive
        .entries()
        .map_err(|e| TarError::InvalidArchive(format!("Failed to read archive entries: {e}")))?
    {
        let entry = entry_result
            .map_err(|e| TarError::InvalidArchive(format!("Failed to read entry: {e}")))?;

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

            let path = entry
                .path()
                .map_err(|e| TarError::InvalidArchive(format!("Failed to read entry path: {e}")))?;

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
            let path = entry
                .path()
                .map_err(|e| TarError::InvalidArchive(format!("Failed to read entry path: {e}")))?;

            println!("{}", path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompressionMode;
    use std::fs;
    use tar::Builder;
    use tempfile::tempdir;

    fn write_zstd_tar(archive_path: &Path) {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size("hello".len() as u64);
            header.set_cksum();
            builder
                .append_data(&mut header, "listed.txt", std::io::Cursor::new("hello"))
                .unwrap();
            builder.finish().unwrap();
        }
        let compressed = zstd::stream::encode_all(std::io::Cursor::new(tar_bytes), 0).unwrap();
        fs::write(archive_path, compressed).unwrap();
    }

    #[test]
    fn test_list_archive_with_zstd_non_verbose() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("archive.tar.zst");
        write_zstd_tar(&archive_path);

        list_archive(&archive_path, false, CompressionMode::Zstd).unwrap();
    }

    #[test]
    fn test_list_archive_with_zstd_verbose() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("archive.tar.zst");
        write_zstd_tar(&archive_path);

        list_archive(&archive_path, true, CompressionMode::Zstd).unwrap();
    }
}
