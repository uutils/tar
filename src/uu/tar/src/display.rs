// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use chrono::{TimeZone, Utc};
use std::io::Write;
use std::path::Path;
use uucore::fs::display_permissions_unix;

/// Print a verbose (ls -l style) line for an entry in a tar archive
pub fn print_entry_verbose<W: Write>(
    mut out: W,
    header: &tar::Header,
    path: &Path,
) -> std::io::Result<()> {
    let mode = header.mode().unwrap_or(0);
    let entry_type = header.entry_type();
    let owner = header
        .username()
        .ok()
        .flatten()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| header.uid().unwrap_or(0).to_string());
    let group = header
        .groupname()
        .ok()
        .flatten()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| header.gid().unwrap_or(0).to_string());
    let size = header.size().unwrap_or(0);
    let mtime = header.mtime().unwrap_or(0);

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

    // TODO: use path.has_trailing_sep() when stable
    let path_str = path.display().to_string();
    let suffix = if entry_type.is_dir()
        && !path_str.ends_with('/')
        && !path_str.ends_with(std::path::MAIN_SEPARATOR)
    {
        std::path::MAIN_SEPARATOR_STR
    } else {
        ""
    };

    writeln!(
        out,
        "{permissions} {owner}/{group} {size:>8} {date_str} {path_str}{suffix}"
    )
}
