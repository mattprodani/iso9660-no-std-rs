// SPDX-License-Identifier: (MIT OR Apache-2.0)

extern crate iso9660;
extern crate md5;

use iso9660::io::Read;
use iso9660::{DirectoryEntry, ISO9660};
use std::fs::File;
use std::io::{self, Read as _, Seek as _};

#[derive(Debug)]
struct MyError(std::io::Error);
impl core::error::Error for MyError {}
impl embedded_io::Error for MyError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}
impl core::fmt::Display for MyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

struct MyFile(File);
impl embedded_io::ErrorType for MyFile {
    type Error = MyError;
}

impl embedded_io::Read for MyFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(buf).map_err(MyError)
    }
}
impl embedded_io::Seek for MyFile {
    fn seek(&mut self, pos: embedded_io::SeekFrom) -> Result<u64, Self::Error> {
        let seek = match pos {
            embedded_io::SeekFrom::Start(i) => io::SeekFrom::Start(i),
            embedded_io::SeekFrom::End(i) => io::SeekFrom::End(i),
            embedded_io::SeekFrom::Current(i) => io::SeekFrom::Current(i),
        };
        self.0.seek(seek).map_err(MyError)
    }
}

#[test]
fn test_dir_joliet() {
    let fs = ISO9660::new(MyFile(File::open("test_joliet.iso").unwrap())).unwrap();

    let mut iter = fs.root.contents();
    assert_eq!(iter.next().unwrap().unwrap().identifier(), ".");
    assert_eq!(iter.next().unwrap().unwrap().identifier(), "..");
    assert_eq!(iter.next().unwrap().unwrap().identifier(), "A");
    assert_eq!(iter.next().unwrap().unwrap().identifier(), "GPL_3_0.TXT");
    assert_eq!(
        iter.next().unwrap().unwrap().identifier(),
        "GPL_LONG_FILENAME.TXT"
    );
    assert!(iter.next().is_none());
}

#[test]
fn test_dir() {
    let fs = ISO9660::new(MyFile(File::open("test.iso").unwrap())).unwrap();

    let mut iter = fs.root.contents();
    assert_eq!(iter.next().unwrap().unwrap().identifier(), ".");
    assert_eq!(iter.next().unwrap().unwrap().identifier(), "..");
    assert_eq!(iter.next().unwrap().unwrap().identifier(), "A");
    assert_eq!(iter.next().unwrap().unwrap().identifier(), "GPL_3_0.TXT");
    assert!(iter.next().is_none());
}

#[test]
fn test_large_file() {
    let fs = ISO9660::new(MyFile(File::open("test.iso").unwrap())).unwrap();

    let file = match fs.open("gpl_3_0.txt").unwrap().unwrap() {
        DirectoryEntry::File(file) => file,
        _ => panic!("Not a file"),
    };

    let mut reader = file.read();
    let mut buf = vec![0; file.size() as usize];
    reader.read(&mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    let hash = md5::compute(text);
    assert_eq!(format!("{:x}", hash), "1ebbd3e34237af26da5dc08a4e440464");
}

#[test]
fn test_extra_slashes() {
    let fs = ISO9660::new(MyFile(File::open("test.iso").unwrap())).unwrap();

    assert!(fs.open("///a/b/c/1").unwrap().is_some());
    assert!(fs.open("a/b/c/1///").unwrap().is_some());
    assert!(fs.open("a/b//c/1").unwrap().is_some());
    assert!(fs.open("/a/b//c////1/").unwrap().is_some());
}

#[test]
fn test_large_dir() {
    let fs = ISO9660::new(MyFile(File::open("test.iso").unwrap())).unwrap();

    let dir = match fs.open("a/b/c").unwrap().unwrap() {
        DirectoryEntry::Directory(dir) => dir,
        _ => panic!("Not a directory"),
    };

    // 200 files, plus '.' and '..'
    assert_eq!(dir.contents().map(Result::unwrap).count(), 202);
    assert_eq!(dir.block_count(), 4);
}
