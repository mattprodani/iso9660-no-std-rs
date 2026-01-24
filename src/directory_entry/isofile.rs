// SPDX-License-Identifier: (MIT OR Apache-2.0)

use alloc::str::FromStr;
use alloc::string::String;
use core::cmp::min;
use core::fmt;
use embedded_io::Read;
use embedded_io::{Seek, SeekFrom, Write};

use time::OffsetDateTime;

use super::DirectoryEntryHeader;
use crate::{FileRef, ISO9660Reader, ISOError};

#[derive(Clone)]
pub struct ISOFile<T: ISO9660Reader> {
    pub header: DirectoryEntryHeader,
    pub identifier: String,
    // File version; ranges from 1 to 32767
    pub version: u16,
    file: FileRef<T>,
}

impl<T: ISO9660Reader> fmt::Debug for ISOFile<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ISOFile")
            .field("header", &self.header)
            .field("identifier", &self.identifier)
            .field("version", &self.version)
            .finish()
    }
}

impl<T: ISO9660Reader> ISOFile<T> {
    pub(crate) fn new(
        header: DirectoryEntryHeader,
        mut identifier: String,
        file: FileRef<T>,
    ) -> Result<ISOFile<T>, ISOError<T::Error>> {
        // Files (not directories) in ISO 9660 have a version number, which is
        // provided at the end of the identifier, seperated by ';'.
        // If not, assume 1.
        let version = match identifier.rfind(';') {
            Some(idx) => {
                let version = u16::from_str(&identifier[idx + 1..])?;
                identifier.truncate(idx);
                version
            }
            None => 1,
        };

        // Files without an extension have a '.' at the end
        if identifier.ends_with('.') {
            identifier.pop();
        }

        Ok(ISOFile {
            header,
            identifier,
            version,
            file,
        })
    }

    pub fn size(&self) -> u32 {
        self.header.extent_length
    }

    pub fn time(&self) -> OffsetDateTime {
        self.header.time
    }

    pub fn read(&self) -> ISOFileReader<T> {
        ISOFileReader {
            buf: [0; 2048],
            buf_lba: None,
            seek: 0,
            start_lba: self.header.extent_loc,
            size: self.size() as usize,
            file: self.file.clone(),
        }
    }
}

pub struct ISOFileReader<T: ISO9660Reader> {
    buf: [u8; 2048],
    buf_lba: Option<u64>,
    seek: usize,
    start_lba: u32,
    size: usize,
    file: FileRef<T>,
}

impl<T: ISO9660Reader> embedded_io::ErrorType for ISOFileReader<T> {
    type Error = T::Error;
}

impl<T: ISO9660Reader> Read for ISOFileReader<T> {
    fn read(&mut self, mut buf: &mut [u8]) -> core::result::Result<usize, T::Error> {
        let mut seek = self.seek;
        while !buf.is_empty() && seek < self.size {
            let lba = self.start_lba as u64 + (seek as u64 / 2048);
            if self.buf_lba != Some(lba) {
                self.file.read_at(&mut self.buf, lba)?;
                self.buf_lba = Some(lba);
            }

            let start = seek % 2048;
            let end = min(self.size - (seek / 2048) * 2048, 2048);
            seek += buf.write(&self.buf[start..end]).unwrap();
        }

        let bytes = seek - self.seek;
        self.seek = seek;
        Ok(bytes)
    }

    // TODO implement `read_buf` on nightly
}

impl<T: ISO9660Reader> Seek for ISOFileReader<T> {
    fn seek(&mut self, pos: SeekFrom) -> core::result::Result<u64, T::Error> {
        let seek = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => self.size as i64 + pos,
            SeekFrom::Current(pos) => self.seek as i64 + pos,
        };

        if seek < 0 {
            Ok(0) // incorrect shld return error.
        } else {
            self.seek = seek as usize;
            Ok(seek as u64)
        }
    }
}
