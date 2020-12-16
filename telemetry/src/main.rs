#[macro_use]
extern crate lazy_static;
use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};

mod parsed;
mod raw;

const CHUNK_SIZE: usize = 1000;

fn main() {
    println!("Parsing raw file (this may take a few seconds)...");
    read_raw_and_parse("log_raw.txt", "log_parsed.txt");
    println!("Done!");
}

/// Reads a raw log file and parses it into another file
pub fn read_raw_and_parse(raw_file: &str, out_file: &str) {
    let file = File::open(raw_file).expect("File not found");
    let reader = BufReader::new(file);

    let out_file = File::create(out_file).expect("Unable to create output log file");
    let mut out_file = LineWriter::new(out_file);

    let mut i = 1;
    for lines in &reader.lines().chunks(CHUNK_SIZE) {
        let lines = lines.map(|x| x.unwrap()).collect::<Vec<_>>();

        for line in lines {
            match raw::parse_raw(&line) {
                Ok(parsed) => {
                    let result = serde_json::to_string(&parsed).unwrap();
                    out_file
                        .write_all(result.as_bytes())
                        .expect("Unable to write line");
                    out_file.write_all(b"\n").unwrap();
                }
                Err(e) => {
                    eprintln!("WARNING: {}. Could not parse line {}", e, line);
                }
            }
        }

        if i % 10 == 0 {
            println!("Read {} lines...", i * CHUNK_SIZE);
        }
        i += 1;
    }
}
