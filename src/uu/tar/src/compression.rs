// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::CompressionMode;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Cursor, Read, Write};

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

pub(crate) fn open_archive_reader<R>(
    input: R,
    mode: CompressionMode,
) -> Result<Box<dyn Read>, TarError>
where
    R: Read + 'static,
{
    let mut input: Box<dyn Read> = Box::new(input);
    let mode = match mode {
        CompressionMode::Auto => detect_compression(&mut input)?,
        other => other,
    };

    let reader: Box<dyn Read> = match mode {
        CompressionMode::Auto | CompressionMode::None => input,
        CompressionMode::Gzip => Box::new(GzDecoder::new(input)),
    };

    Ok(reader)
}

pub(crate) struct ArchiveWriter<W: Write> {
    inner: ArchiveWriterInner<W>,
}

enum ArchiveWriterInner<W: Write> {
    Plain(W),
    Gzip(GzEncoder<W>),
}

impl<W: Write> ArchiveWriter<W> {
    pub(crate) fn new(output: W, mode: CompressionMode) -> Result<Self, TarError> {
        let inner = match mode {
            CompressionMode::Auto => {
                return Err(TarError::TarOperationError(
                    "internal error: automatic compression is not valid for archive creation"
                        .to_string(),
                ));
            }
            CompressionMode::None => ArchiveWriterInner::Plain(output),
            CompressionMode::Gzip => {
                ArchiveWriterInner::Gzip(GzEncoder::new(output, flate2::Compression::default()))
            }
        };

        Ok(Self { inner })
    }

    pub(crate) fn finish(self) -> Result<(), TarError> {
        match self.inner {
            ArchiveWriterInner::Plain(mut file) => file.flush().map_err(TarError::from),
            ArchiveWriterInner::Gzip(encoder) => {
                let mut output = encoder.finish().map_err(TarError::CannotFinalizeArchive)?;
                output.flush().map_err(TarError::from)
            }
        }
    }
}

impl<W: Write> Write for ArchiveWriter<W> {
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

fn detect_compression(input: &mut Box<dyn Read>) -> Result<CompressionMode, TarError> {
    let mut magic = [0u8; 2];
    let n = input.read(&mut magic).map_err(TarError::Io)?;
    let prefix = Cursor::new(magic[..n].to_vec());
    let rest = std::mem::replace(input, Box::new(std::io::empty()));
    *input = Box::new(prefix.chain(rest));

    if n >= GZIP_MAGIC.len() && magic[..GZIP_MAGIC.len()] == GZIP_MAGIC {
        return Ok(CompressionMode::Gzip);
    }
    Ok(CompressionMode::None)
}
