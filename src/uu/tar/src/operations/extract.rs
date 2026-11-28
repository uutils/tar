// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::compression::open_archive_reader;
use crate::errors::{finish_after_previous_errors, report_recoverable, TarError};
use crate::CompressionMode;
use std::collections::HashSet;
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use tar::{Archive, Entry, EntryType};
use uucore::error::UResult;

/// Extract files from a tar archive
///
/// # Arguments
///
/// * `verbose` - Whether to print verbose output during extraction
/// * `to_stdout` - Write the contents of regular members to standard output
///   instead of creating files (`-O`)
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be opened
/// - The archive format is invalid
/// - Files cannot be extracted due to I/O or permission errors
pub fn extract_archive(
    input: impl Read,
    verbose: bool,
    to_stdout: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(input, compression)?;
    let mut archive = Archive::new(reader);
    if to_stdout {
        return extract_to_stdout(&mut archive, verbose);
    }
    let mut out = BufWriter::new(io::stdout().lock());

    // Members that were refused are reported as they are met; like GNU tar, the
    // remaining members are still extracted and the exit status records the failure.
    let mut refused_member = false;

    // Like GNU tar, directories get their permissions after their contents
    // are extracted, or a read-only directory could not be filled.
    let mut directories = Vec::new();

    for_each_member(&mut archive, verbose, &mut out, |mut entry, path, out| {
        // Directories with a `..` component are not deferred, so they are
        // refused in archive order
        if entry.header().entry_type() == EntryType::Directory
            && !path.components().any(|c| c == Component::ParentDir)
        {
            directories.push((path, entry));
        } else {
            refused_member |= unpack_entry(&mut entry, path, out)?;
        }
        Ok(())
    })?;

    // Children before parents, so a read-only parent is restored last
    directories.sort_by(|a, b| b.0.cmp(&a.0));
    for (path, mut entry) in directories {
        refused_member |= unpack_entry(&mut entry, path, &mut out)?;
    }

    out.flush().map_err(TarError::Io)?;
    if refused_member {
        return finish_after_previous_errors();
    }
    Ok(())
}

/// Like GNU tar's `-O`: the data of regular members goes to standard output
/// and nothing is created on disk. Member names (with `-v`) and warnings go to
/// standard error, so they don't mix with the contents.
fn extract_to_stdout<R: Read>(archive: &mut Archive<R>, verbose: bool) -> UResult<()> {
    let mut data = io::stdout().lock();
    let mut names = io::stderr().lock();
    let mut refused_member = false;

    for_each_member(archive, verbose, &mut names, |mut entry, path, _| {
        // GNU tar refuses these even when nothing is written to disk
        if path.components().any(|c| c == Component::ParentDir) {
            report_recoverable(&TarError::MemberNameContainsDotDot { path })?;
            refused_member = true;
            return Ok(());
        }

        // Like GNU tar, write the data of regular, contiguous and GNU sparse
        // members (tar-rs fills in the holes of the latter). Directories,
        // links and special files have no data to write.
        let kind = entry.header().entry_type();
        if kind.is_file() || kind.is_contiguous() || kind.is_gnu_sparse() {
            io::copy(&mut entry, &mut data).map_err(TarError::Io)?;
            // Flush after each member so `-v` names and contents interleave
            data.flush().map_err(TarError::Io)?;
        }
        Ok(())
    })?;

    if refused_member {
        return finish_after_previous_errors();
    }
    Ok(())
}

/// Reads each member of the archive, warns about leading slashes in its name,
/// prints the name with `-v`, then passes the member and its name to `handle`.
/// Messages and names go to `out`, which `handle` also gets.
fn for_each_member<'a, R: Read, W: Write>(
    archive: &'a mut Archive<R>,
    verbose: bool,
    out: &mut W,
    mut handle: impl FnMut(Entry<'a, R>, PathBuf, &mut W) -> UResult<()>,
) -> UResult<()> {
    let mut warned_slashes = HashSet::new();
    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let entry = entry_result.map_err(TarError::CannotReadEntry)?;
        let path = entry
            .path()
            .map_err(TarError::CannotReadEntryPath)?
            .to_path_buf();

        warn_leading_slashes(&path, &mut warned_slashes, out)?;
        if verbose {
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }
        handle(entry, path, out)?;
    }
    Ok(())
}

/// `unpack_in` drops leading slashes from a member name. Like GNU tar, say so
/// once for each different number of them.
fn warn_leading_slashes(
    path: &Path,
    warned: &mut HashSet<usize>,
    out: &mut impl Write,
) -> UResult<()> {
    let slashes = path
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .take_while(|&&b| b == b'/')
        .count();
    if slashes > 0 && warned.insert(slashes) {
        out.flush().map_err(TarError::Io)?;
        super::warn_removed_prefix("/".repeat(slashes)).map_err(TarError::Io)?;
    }
    Ok(())
}

/// Unpacks one entry into the current directory. Returns true when the entry
/// was refused, which has already been reported.
fn unpack_entry<R: Read>(
    entry: &mut Entry<R>,
    path: PathBuf,
    out: &mut impl Write,
) -> UResult<bool> {
    // `unpack_in` skips, and returns false for, a member whose name contains
    // a `..` component, as it would escape the target directory.
    let unpacked = entry.unpack_in(".").map_err(|e| TarError::CannotExtract {
        path: path.clone(),
        source: e,
    })?;
    if !unpacked {
        out.flush().map_err(TarError::Io)?;
        report_recoverable(&TarError::MemberNameContainsDotDot { path })?;
    }
    Ok(!unpacked)
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
