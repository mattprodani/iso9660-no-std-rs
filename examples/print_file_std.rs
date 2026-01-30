//! Example code for using the 'std' feature flag that maintains
//! same API as iso9660.
// SPDX-License-Identifier: (MIT OR Apache-2.0)

extern crate iso9660;

use embedded_io::Read;
use std::fs::File;
use std::io::{self, Read as _, Seek as _, Write as _};
use std::{env, process};

use iso9660::{DirectoryEntry, ISO9660};

fn main() {
    let args = env::args();

    if args.len() != 3 {
        eprintln!("Requires 2 arguments.");
        process::exit(1);
    }

    let iso_path = env::args().nth(1).unwrap();
    let file_path = env::args().nth(2).unwrap();

    let file = File::open(iso_path).unwrap();
    let fs = ISO9660::new(file).unwrap();

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
