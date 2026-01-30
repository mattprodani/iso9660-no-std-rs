// SPDX-License-Identifier: (MIT OR Apache-2.0)

extern crate iso9660;

use std::fs::File;
use std::io::{self, Read as _, Seek as _};
use std::{env, process};

use iso9660::{DirectoryEntry, ISO9660Reader, ISODirectory, ISO9660};

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

fn main() {
    let args = env::args();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Requires 1 or 2 arguments.");
        process::exit(1);
    }

    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let dirpath = args.next();

    let file = File::open(path).unwrap();
    let fs = ISO9660::new(MyFile(file)).unwrap();

    if let Some(dirpath) = dirpath {
        match fs.open(&dirpath).unwrap() {
            Some(DirectoryEntry::Directory(dir)) => {
                print_tree(&dir, 0);
            }
            Some(DirectoryEntry::File(_)) => {
                eprintln!("'{}' is not a directory", dirpath);
                process::exit(1);
            }
            None => {
                eprintln!("'{}' does not exist", dirpath);
                process::exit(1);
            }
        }
    } else {
        print_tree(&fs.root, 0);
    }
}

fn print_tree<T: ISO9660Reader>(dir: &ISODirectory<T>, level: u32) {
    for entry in dir.contents() {
        match entry.unwrap() {
            DirectoryEntry::Directory(dir) => {
                if dir.identifier == "." || dir.identifier == ".." {
                    continue;
                }
                for _i in 0..level {
                    print!("  ");
                }
                println!("- {}/", dir.identifier);
                print_tree(&dir, level + 1);
            }
            DirectoryEntry::File(file) => {
                for _i in 0..level {
                    print!("  ");
                }
                println!("- {}", file.identifier);
            }
        }
    }
}
