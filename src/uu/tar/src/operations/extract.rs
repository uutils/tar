// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::operations::compression::open_archive_reader;
use crate::CompressionMode;
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
pub fn extract_archive(
    archive_path: &Path,
    verbose: bool,
    compression: CompressionMode,
) -> UResult<()> {
    let reader = open_archive_reader(archive_path, compression)?;
    let mut archive = Archive::new(reader);

    // Extract to current directory
    if verbose {
        println!("Extracting archive: {}", archive_path.display());
    }

    // Iterate through entries for verbose output and error handling
    for entry_result in archive
        .entries()
        .map_err(|e| TarError::InvalidArchive(format!("Failed to read archive entries: {e}")))?
    {
        let mut entry = entry_result
            .map_err(|e| TarError::InvalidArchive(format!("Failed to read entry: {e}")))?;

        // Get the path before unpacking (clone it so we can use it after borrowing entry mutably)
        let path = entry
            .path()
            .map_err(|e| TarError::InvalidArchive(format!("Failed to read entry path: {e}")))?
            .to_path_buf();

        if verbose {
            println!("{}", path.display());
        }

        // Unpack the entry
        entry.unpack_in(".").map_err(|e| {
            TarError::TarOperationError(format!("Failed to extract '{}': {}", path.display(), e))
        })?;
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
        let result = extract_archive(&archive_path, true, CompressionMode::Zstd);

        result.unwrap();
        assert_eq!(
            fs::read_to_string(tempdir.path().join("extracted.txt")).unwrap(),
            "hello"
        );
    }
}
