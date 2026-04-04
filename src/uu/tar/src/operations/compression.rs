// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::CompressionMode;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn open_archive_reader(
    archive_path: &Path,
    compression: CompressionMode,
) -> Result<Box<dyn Read>, TarError> {
    let file = File::open(archive_path).map_err(|e| TarError::from_io_error(e, archive_path))?;

    match compression {
        CompressionMode::None => Ok(Box::new(file)),
        CompressionMode::Zstd => {
            let decoder = zstd::stream::read::Decoder::new(file).map_err(|e| {
                TarError::InvalidArchive(format!(
                    "Failed to initialize zstd decoder for '{}': {}",
                    archive_path.display(),
                    e
                ))
            })?;
            Ok(Box::new(decoder))
        }
    }
}

pub struct ArchiveWriter {
    inner: ArchiveWriterInner,
}

enum ArchiveWriterInner {
    Plain(File),
    Zstd(zstd::stream::write::Encoder<'static, File>),
}

impl ArchiveWriter {
    pub fn create(archive_path: &Path, compression: CompressionMode) -> Result<Self, TarError> {
        let file = File::create(archive_path).map_err(|e| {
            TarError::TarOperationError(format!(
                "Cannot create archive '{}': {}",
                archive_path.display(),
                e
            ))
        })?;

        let inner = match compression {
            CompressionMode::None => ArchiveWriterInner::Plain(file),
            CompressionMode::Zstd => {
                let encoder = zstd::stream::write::Encoder::new(file, 0).map_err(|e| {
                    TarError::TarOperationError(format!(
                        "Failed to initialize zstd encoder for '{}': {}",
                        archive_path.display(),
                        e
                    ))
                })?;
                ArchiveWriterInner::Zstd(encoder)
            }
        };

        Ok(Self { inner })
    }

    pub fn finish(self) -> Result<(), TarError> {
        match self.inner {
            ArchiveWriterInner::Plain(mut file) => file.flush().map_err(TarError::from),
            ArchiveWriterInner::Zstd(encoder) => encoder.finish().map(|_| ()).map_err(|e| {
                TarError::TarOperationError(format!("Failed to finalize zstd archive: {e}"))
            }),
        }
    }
}

impl Write for ArchiveWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            ArchiveWriterInner::Plain(file) => file.write(buf),
            ArchiveWriterInner::Zstd(encoder) => encoder.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            ArchiveWriterInner::Plain(file) => file.flush(),
            ArchiveWriterInner::Zstd(encoder) => encoder.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use tempfile::tempdir;

    #[test]
    fn test_plain_archive_writer_and_reader() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("plain.tar");

        let mut writer = ArchiveWriter::create(&archive_path, CompressionMode::None).unwrap();
        writer.write_all(b"plain data").unwrap();
        writer.flush().unwrap();
        writer.finish().unwrap();

        let mut reader = open_archive_reader(&archive_path, CompressionMode::None).unwrap();
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, b"plain data");
    }

    #[test]
    fn test_zstd_archive_writer_and_reader() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("archive.tar.zst");

        let mut writer = ArchiveWriter::create(&archive_path, CompressionMode::Zstd).unwrap();
        writer.write_all(b"zstd data").unwrap();
        writer.flush().unwrap();
        writer.finish().unwrap();

        let mut reader = open_archive_reader(&archive_path, CompressionMode::Zstd).unwrap();
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, b"zstd data");
    }

    #[test]
    fn test_open_archive_reader_missing_file() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("missing.tar.zst");

        let err = open_archive_reader(&archive_path, CompressionMode::Zstd)
            .err()
            .unwrap();
        assert!(matches!(err, TarError::FileNotFound(_)));
    }

    #[test]
    fn test_open_archive_reader_invalid_zstd_stream_fails_on_read() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("invalid.tar.zst");
        std::fs::write(&archive_path, b"not zstd").unwrap();

        let mut reader = open_archive_reader(&archive_path, CompressionMode::Zstd).unwrap();
        let mut contents = Vec::new();
        assert!(reader.read_to_end(&mut contents).is_err());
    }
}
