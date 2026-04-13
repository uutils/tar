// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::CompressionMode;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

pub(crate) fn open_archive_reader(
    archive_path: &Path,
    mode: CompressionMode,
) -> Result<Box<dyn Read>, TarError> {
    let mut file =
        File::open(archive_path).map_err(|e| TarError::from_io_error(e, archive_path))?;
    let mode = match mode {
        CompressionMode::Auto => detect_compression(&mut file)?,
        other => other,
    };

    let reader: Box<dyn Read> = match mode {
        CompressionMode::Auto | CompressionMode::None => Box::new(file),
        CompressionMode::Gzip => Box::new(GzDecoder::new(file)),
    };

    Ok(reader)
}

pub(crate) struct ArchiveWriter {
    inner: ArchiveWriterInner,
}

enum ArchiveWriterInner {
    Plain(File),
    Gzip(GzEncoder<File>),
}

impl ArchiveWriter {
    pub(crate) fn create(archive_path: &Path, mode: CompressionMode) -> Result<Self, TarError> {
        let file = File::create(archive_path).map_err(|e| TarError::CannotCreateArchive {
            path: archive_path.to_path_buf(),
            source: e,
        })?;

        let inner = match mode {
            CompressionMode::Auto => {
                return Err(TarError::TarOperationError(
                    "internal error: automatic compression is not valid for archive creation"
                        .to_string(),
                ));
            }
            CompressionMode::None => ArchiveWriterInner::Plain(file),
            CompressionMode::Gzip => {
                ArchiveWriterInner::Gzip(GzEncoder::new(file, flate2::Compression::default()))
            }
        };

        Ok(Self { inner })
    }

    pub(crate) fn finish(self) -> Result<(), TarError> {
        match self.inner {
            ArchiveWriterInner::Plain(mut file) => file.flush().map_err(TarError::from),
            ArchiveWriterInner::Gzip(encoder) => encoder
                .finish()
                .map(|_| ())
                .map_err(TarError::CannotFinalizeArchive),
        }
    }
}

impl Write for ArchiveWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            ArchiveWriterInner::Plain(file) => file.write(buf),
            ArchiveWriterInner::Gzip(encoder) => encoder.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            ArchiveWriterInner::Plain(file) => file.flush(),
            ArchiveWriterInner::Gzip(encoder) => encoder.flush(),
        }
    }
}

fn detect_compression(file: &mut File) -> Result<CompressionMode, TarError> {
    let mut magic = [0u8; 2];
    let n = file.read(&mut magic).map_err(TarError::Io)?;
    file.seek(std::io::SeekFrom::Start(0))
        .map_err(TarError::Io)?;

    if n >= GZIP_MAGIC.len() && magic[..GZIP_MAGIC.len()] == GZIP_MAGIC {
        return Ok(CompressionMode::Gzip);
    }
    Ok(CompressionMode::None)
}
