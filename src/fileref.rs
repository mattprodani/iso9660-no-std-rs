// SPDX-License-Identifier: (MIT OR Apache-2.0)

use crate::io::{ErrorType, Read, Seek, SeekFrom};
use alloc::rc::Rc;
use core::cell::RefCell;

pub trait ISO9660Reader: ErrorType {
    /// Read the block(s) at a given LBA (logical block address)
    fn read_at(&mut self, buf: &mut [u8], lba: u64) -> Result<usize, ReaderError!(Self)>;
}

impl<T: Read + Seek> ISO9660Reader for T {
    fn read_at(&mut self, buf: &mut [u8], lba: u64) -> Result<usize, ReaderError!(Self)> {
        self.seek(SeekFrom::Start(lba * 2048))?;
        self.read(buf)
    }
}

// TODO: Figure out if sane API possible without Rc/RefCell
pub(crate) struct FileRef<T: ISO9660Reader>(Rc<RefCell<T>>);

impl<T: ISO9660Reader> Clone for FileRef<T> {
    fn clone(&self) -> FileRef<T> {
        FileRef(self.0.clone())
    }
}

impl<T: ISO9660Reader> FileRef<T> {
    pub fn new(reader: T) -> FileRef<T> {
        FileRef(Rc::new(RefCell::new(reader)))
    }

    /// Read the block(s) at a given LBA (logical block address)
    pub fn read_at(&self, buf: &mut [u8], lba: u64) -> Result<usize, ReaderError!(T)> {
        (*self.0).borrow_mut().read_at(buf, lba)
    }
}
