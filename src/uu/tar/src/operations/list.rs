// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::compression::open_archive_reader;
use crate::CompressionMode;
use chrono::{TimeZone, Utc};
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;
use uucore::fs::display_permissions_unix;

/// List the contents of a tar archive, printing one entry per line.
pub fn list_archive(
    input: impl Read,
    archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(input, archive_path, compression)?;
    let mut archive = Archive::new(reader);
    let mut out = BufWriter::new(io::stdout().lock());

    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let entry = entry_result.map_err(TarError::CannotReadEntry)?;

        if verbose {
            let formatted = format_verbose_entry(&entry)?;
            writeln!(out, "{formatted}").map_err(TarError::Io)?;
        } else {
            let path = entry.path().map_err(TarError::CannotReadEntryPath)?;
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}

fn format_verbose_entry<R: Read>(entry: &tar::Entry<'_, R>) -> Result<String, TarError> {
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

    Ok(format!(
        "{permissions} {owner}/{group} {size:>8} {date_str} {}",
        path.display()
    ))
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

        let input = fs::File::open(&archive_path).unwrap();
        list_archive(input, &archive_path, false, CompressionMode::Zstd).unwrap();
    }

    #[test]
    fn test_list_archive_with_zstd_verbose() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("archive.tar.zst");
        write_zstd_tar(&archive_path);

        let input = fs::File::open(&archive_path).unwrap();
        list_archive(input, &archive_path, true, CompressionMode::Zstd).unwrap();
    }
}
