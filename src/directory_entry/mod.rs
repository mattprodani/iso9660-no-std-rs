// SPDX-License-Identifier: (MIT OR Apache-2.0)

pub use self::isodirectory::ISODirectory;
pub use self::isofile::ISOFile;

use crate::parse::DirectoryEntryReader;
use crate::parse::{DirectoryEntryHeader, FileFlags};
use crate::{FileRef, ISO9660Reader, ISOError};
use alloc::string::String;

mod isodirectory;
mod isofile;

#[derive(Clone, Debug)]
pub enum DirectoryEntry<T: ISO9660Reader> {
    Directory(ISODirectory<T>),
    File(ISOFile<T>),
}

impl<T: ISO9660Reader> DirectoryEntry<T> {
    pub(crate) fn new(
        header: DirectoryEntryHeader,
        identifier: String,
        file: FileRef<T>,
        reader: DirectoryEntryReader,
    ) -> Result<Self, ISOError<ReaderError!(T)>> {
        if header.file_flags.contains(FileFlags::DIRECTORY) {
            Ok(DirectoryEntry::Directory(ISODirectory::new(
                header, identifier, file, reader,
            )))
        } else {
            Ok(DirectoryEntry::File(ISOFile::new(
                header, identifier, file,
            )?))
        }
    }

    pub fn header(&self) -> &DirectoryEntryHeader {
        match *self {
            DirectoryEntry::Directory(ref dir) => &dir.header,
            DirectoryEntry::File(ref file) => &file.header,
        }
    }

    pub fn identifier(&self) -> &str {
        match *self {
            DirectoryEntry::Directory(ref dir) => &dir.identifier,
            DirectoryEntry::File(ref file) => &file.identifier,
        }
    }
}
