// SPDX-License-Identifier: (MIT OR Apache-2.0)
extern crate iso9660;

use embedded_io::Read;
use std::fs::File;
use std::io::{self, Read as _, Seek as _, Write as _};
use std::{env, process};

use iso9660::{DirectoryEntry, ISO9660};

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

#[cfg(not(feature = "std"))]
fn main() {
    let args = env::args();

    if args.len() != 3 {
        eprintln!("Requires 2 arguments.");
        process::exit(1);
    }

    let iso_path = env::args().nth(1).unwrap();
    let file_path = env::args().nth(2).unwrap();

    let file = File::open(iso_path).unwrap();
    let fs = ISO9660::new(MyFile(file)).unwrap();

    match fs.open(&file_path).unwrap() {
        Some(DirectoryEntry::File(file)) => {
            let mut stdout = io::stdout();
            let mut buf = vec![0; file.size() as usize];
            let mut reader = file.read();
            dbg!(reader.read(&mut buf).unwrap());

            stdout.write_all(&buf).unwrap();
        }
        Some(_) => panic!("{} is not a file.", file_path),
        None => panic!("'{}' not found", file_path),
    }
}
