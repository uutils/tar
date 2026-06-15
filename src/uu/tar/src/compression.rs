// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use crate::CompressionMode;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{BufReader, Cursor, Read, Write};

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

pub(crate) fn open_archive_reader<'a, R>(
    input: R,
    mode: CompressionMode,
) -> Result<Box<dyn Read + 'a>, TarError>
where
    R: Read + 'a,
{
    let mut input: Box<dyn Read + 'a> = Box::new(input);
    let mode = match mode {
        CompressionMode::Auto => detect_compression(&mut input)?,
        other => other,
    };

    let reader: Box<dyn Read + 'a> = match mode {
        CompressionMode::Auto | CompressionMode::None => Box::new(BufReader::new(input)),
        CompressionMode::Gzip => Box::new(GzDecoder::new(input)),
        CompressionMode::Zstd => Box::new(zstd::stream::read::Decoder::new(input)?),
    };

    Ok(reader)
}

pub(crate) struct ArchiveWriter<W: Write> {
    inner: ArchiveWriterInner<W>,
}

enum ArchiveWriterInner<W: Write> {
    Plain(W),
    Gzip(GzEncoder<W>),
    Zstd(zstd::stream::write::Encoder<'static, W>),
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
            CompressionMode::Zstd => {
                let encoder = zstd::stream::write::Encoder::new(output, 0)?;
                ArchiveWriterInner::Zstd(encoder)
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
            ArchiveWriterInner::Zstd(encoder) => {
                let mut output = encoder.finish()?;
                output.flush().map_err(TarError::Io)
            }
        }
    }
}

impl<W: Write> Write for ArchiveWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            ArchiveWriterInner::Plain(file) => file.write(buf),
            ArchiveWriterInner::Gzip(encoder) => encoder.write(buf),
            ArchiveWriterInner::Zstd(encoder) => encoder.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            ArchiveWriterInner::Plain(file) => file.flush(),
            ArchiveWriterInner::Gzip(encoder) => encoder.flush(),
            ArchiveWriterInner::Zstd(encoder) => encoder.flush(),
        }
    }
}

fn detect_compression(input: &mut Box<dyn Read + '_>) -> Result<CompressionMode, TarError> {
    let mut magic = [0u8; 2];
    let mut n = 0;
    while n < magic.len() {
        match input.read(&mut magic[n..]) {
            Ok(0) => break,
            Ok(read) => n += read,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(TarError::Io(e)),
        }
    }

    let prefix = Cursor::new(magic[..n].to_vec());
    let rest = std::mem::replace(input, Box::new(std::io::empty()));
    *input = Box::new(prefix.chain(rest));

    if n >= GZIP_MAGIC.len() && magic[..GZIP_MAGIC.len()] == GZIP_MAGIC {
        return Ok(CompressionMode::Gzip);
    }
    Ok(CompressionMode::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    struct OneByteReader<R> {
        inner: R,
    }

    impl<R: Read> Read for OneByteReader<R> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let len = buf.len().min(1);
            self.inner.read(&mut buf[..len])
        }
    }

    struct InterruptedOnceReader<R> {
        inner: R,
        interrupted: bool,
    }

    impl<R: Read> Read for InterruptedOnceReader<R> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            self.inner.read(buf)
        }
    }

    fn gzip_bytes(payload: &[u8]) -> Vec<u8> {
        let mut gzipped = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gzipped, flate2::Compression::default());
            encoder.write_all(payload).unwrap();
            encoder.finish().unwrap();
        }
        gzipped
    }

    #[test]
    fn auto_detects_gzip_after_short_magic_read() {
        let gzipped = gzip_bytes(b"payload");

        let mut reader = open_archive_reader(
            OneByteReader {
                inner: Cursor::new(gzipped),
            },
            CompressionMode::Auto,
        )
        .unwrap();

        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, b"payload");
    }

    #[test]
    fn auto_detect_retries_interrupted_magic_read() {
        let mut reader = open_archive_reader(
            InterruptedOnceReader {
                inner: Cursor::new(gzip_bytes(b"payload")),
                interrupted: false,
            },
            CompressionMode::Auto,
        )
        .unwrap();

        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, b"payload");
    }

    #[test]
    fn auto_detect_preserves_empty_input() {
        let mut reader =
            open_archive_reader(Cursor::new(Vec::new()), CompressionMode::Auto).unwrap();

        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).unwrap();
        assert!(contents.is_empty());
    }

    #[test]
    fn archive_writer_rejects_auto_compression_for_creation() {
        let err = ArchiveWriter::new(io::sink(), CompressionMode::Auto)
            .err()
            .expect("auto compression should not be valid for archive creation");

        match err {
            TarError::TarOperationError(message) => {
                assert!(message.contains("automatic compression is not valid"));
            }
            other => panic!("expected tar operation error, got {other:?}"),
        }
    }

    #[test]
    fn archive_writer_flushes_plain_output() {
        let mut writer = ArchiveWriter::new(io::sink(), CompressionMode::None).unwrap();
        writer.write_all(b"payload").unwrap();
        writer.flush().unwrap();
        writer.finish().unwrap();
    }

    #[test]
    fn archive_writer_flushes_gzip_output() {
        let mut writer = ArchiveWriter::new(io::sink(), CompressionMode::Gzip).unwrap();
        writer.write_all(b"payload").unwrap();
        writer.flush().unwrap();
        writer.finish().unwrap();
    }

    #[test]
    fn archive_writer_flushes_zstd_output() {
        let mut writer = ArchiveWriter::new(io::sink(), CompressionMode::Zstd).unwrap();
        writer.write_all(b"payload").unwrap();
        writer.flush().unwrap();
        writer.finish().unwrap();
    }
}
