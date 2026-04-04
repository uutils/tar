// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::compression::open_archive_reader;
use crate::CompressionMode;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use tar::Archive;
use uucore::error::UResult;

/// Extract files from a tar archive
///
/// # Arguments
///
/// * `archive_path` - Path to the tar archive to extract
/// * `verbose` - Whether to print verbose output during extraction
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be opened
/// - The archive format is invalid
/// - Files cannot be extracted due to I/O or permission errors
pub(crate) fn extract_archive(
    input: impl Read,
    archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(input, archive_path, compression)?;
    let mut archive = Archive::new(reader);
    let mut out = BufWriter::new(io::stdout().lock());

    // Extract to current directory
    if verbose {
        writeln!(out, "Extracting archive: {}", archive_path.display()).map_err(TarError::Io)?;
    }

    // Iterate through entries for verbose output and error handling
    for entry_result in archive.entries().map_err(TarError::CannotReadEntries)? {
        let mut entry = entry_result.map_err(TarError::CannotReadEntry)?;

        // Get the path before unpacking (clone it so we can use it after borrowing entry mutably)
        let path = entry
            .path()
            .map_err(TarError::CannotReadEntryPath)?
            .to_path_buf();

        if verbose {
            writeln!(out, "{}", path.display()).map_err(TarError::Io)?;
        }

        // Unpack the entry
        entry.unpack_in(".").map_err(|e| TarError::CannotExtract {
            path: path.clone(),
            source: e,
        })?;
    }

    out.flush().map_err(TarError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompressionMode;
    use std::fs;
    use tar::Builder;
    use tempfile::tempdir;

    #[test]
    fn test_extract_archive_with_zstd() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("archive.tar.zst");

        let mut tar_bytes = Vec::new();
        {
            let mut builder = Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size("hello".len() as u64);
            header.set_cksum();
            builder
                .append_data(&mut header, "extracted.txt", std::io::Cursor::new("hello"))
                .unwrap();
            builder.finish().unwrap();
        }
        let compressed = zstd::stream::encode_all(std::io::Cursor::new(tar_bytes), 0).unwrap();
        fs::write(&archive_path, compressed).unwrap();

        let _guard = crate::operations::TestDirGuard::enter(tempdir.path());
        let input = fs::File::open(&archive_path).unwrap();
        let result = extract_archive(input, &archive_path, true, CompressionMode::Zstd);

        result.unwrap();
        assert_eq!(
            fs::read_to_string(tempdir.path().join("extracted.txt")).unwrap(),
            "hello"
        );
    }
}
