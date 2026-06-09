// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::CompressionMode;
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub(crate) fn open_archive_reader<'a, R: Read + 'a>(
    input: R,
    archive_path: &Path,
    compression: CompressionMode,
) -> Result<Box<dyn Read + 'a>, TarError> {
    match compression {
        CompressionMode::None => Ok(Box::new(BufReader::new(input))),
        CompressionMode::Zstd => {
            let decoder = zstd::stream::read::Decoder::new(input).map_err(|e| {
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

pub(crate) struct ArchiveWriter<W: Write> {
    inner: ArchiveWriterInner<W>,
}

enum ArchiveWriterInner<W: Write> {
    Plain(W),
    Zstd(zstd::stream::write::Encoder<'static, W>),
}

impl<W: Write> ArchiveWriter<W> {
    pub(crate) fn new(output: W, compression: CompressionMode) -> Result<Self, TarError> {
        let inner = match compression {
            CompressionMode::None => ArchiveWriterInner::Plain(output),
            CompressionMode::Zstd => {
                let encoder = zstd::stream::write::Encoder::new(output, 0).map_err(|e| {
                    TarError::TarOperationError(format!("Failed to initialize zstd encoder: {e}"))
                })?;
                ArchiveWriterInner::Zstd(encoder)
            }
        };

        Ok(Self { inner })
    }

    pub fn finish(self) -> Result<(), TarError> {
        match self.inner {
            ArchiveWriterInner::Plain(mut output) => output.flush().map_err(TarError::Io),
            ArchiveWriterInner::Zstd(encoder) => encoder.finish().map(|_| ()).map_err(|e| {
                TarError::TarOperationError(format!("Failed to finalize zstd archive: {e}"))
            }),
        }
    }
}

impl<W: Write> Write for ArchiveWriter<W> {
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

        let output = std::fs::File::create(&archive_path).unwrap();
        let mut writer = ArchiveWriter::new(output, CompressionMode::None).unwrap();
        writer.write_all(b"plain data").unwrap();
        writer.flush().unwrap();
        writer.finish().unwrap();

        let input = std::fs::File::open(&archive_path).unwrap();
        let mut reader = open_archive_reader(input, &archive_path, CompressionMode::None).unwrap();
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, b"plain data");
    }

    #[test]
    fn test_zstd_archive_writer_and_reader() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("archive.tar.zst");

        let output = std::fs::File::create(&archive_path).unwrap();
        let mut writer = ArchiveWriter::new(output, CompressionMode::Zstd).unwrap();
        writer.write_all(b"zstd data").unwrap();
        writer.flush().unwrap();
        writer.finish().unwrap();

        let input = std::fs::File::open(&archive_path).unwrap();
        let mut reader = open_archive_reader(input, &archive_path, CompressionMode::Zstd).unwrap();
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, b"zstd data");
    }

    #[test]
    fn test_open_archive_reader_invalid_zstd_stream_fails_on_read() {
        let tempdir = tempdir().unwrap();
        let archive_path = tempdir.path().join("invalid.tar.zst");
        std::fs::write(&archive_path, b"not zstd").unwrap();

        let input = std::fs::File::open(&archive_path).unwrap();
        let mut reader = open_archive_reader(input, &archive_path, CompressionMode::Zstd).unwrap();
        let mut contents = Vec::new();
        assert!(reader.read_to_end(&mut contents).is_err());
    }
}
