// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::display;
use crate::errors::TarError;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;

/// List the contents of a tar archive, printing one entry per line.
pub fn list_archive(archive_path: &Path, verbose: u8) -> UResult<()> {
    let file: File =
        File::open(archive_path).map_err(|e| TarError::from_io_error(e, archive_path))?;
    let mut archive = Archive::new(file);
    let mut out = BufWriter::new(io::stdout().lock());

    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let entry = entry_result.map_err(TarError::CannotReadEntry)?;
        let path = entry.path().map_err(TarError::CannotReadEntryPath)?;

        if verbose >= 1 {
            display::print_entry_verbose(&mut out, entry.header(), &path).map_err(TarError::Io)?;
        } else {
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}
