#[macro_use]
extern crate lazy_static;
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use regex::Regex;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};

const CHUNK_SIZE: usize = 1000;

fn main() {
    let file = File::open("log_raw.txt").expect("File not found");
    let reader = BufReader::new(file);

    let out_file = File::create("log_parsed.txt").expect("Unable to create output log file");
    let mut out_file = LineWriter::new(out_file);

    for lines in &reader.lines().chunks(CHUNK_SIZE) {
        let lines = lines.map(|x| x.unwrap()).collect::<Vec<_>>();
        let result = parse_raw(lines);
        for r in result {
            out_file
                .write_all(r.as_bytes())
                .expect("Unable to write line");
            out_file.write_all(b"\n").unwrap();
        }
    }
}

/// Parses a raw log file into one that has the delta time between strokes and extraneous
/// information (such as the output) removed.
///
/// Delta that is too large is represented by 2^16-1 (max of u16)
fn parse_raw(raw: Vec<String>) -> Vec<String> {
    let mut prev_time: Option<DateTime<Utc>> = None;

    let mut result = Vec::with_capacity(raw.len());

    for line in raw {
        // ignore all none text strokes for now
        if !line.contains("[Replace(") {
            continue;
        }

        if let Some((time, stroke)) = parse_line(&line) {
            // get difference between current stroke and previous stroke's time
            let diff = if let Some(prev_datetime) = prev_time {
                time.signed_duration_since(prev_datetime)
            } else {
                // the first stroke has duration 0 since previous stroke
                Duration::zero()
            };
            prev_time = Some(time);

            if diff.num_milliseconds() < 0 {
                panic!("The log file has gone backwards! At: {}", line);
            }

            // convert the time difference to u16 (and cap it at 2^16-1)
            let delta = u16::try_from(diff.num_milliseconds()).unwrap_or(u16::MAX);

            result.push(format!("{: >5},{}", delta, stroke));
        } else {
            eprintln!("WARNING: Could not parse line {}", line);
        }
    }

    result
}

type Stroke = String;
fn parse_line(line: &str) -> Option<(DateTime<Utc>, Stroke)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^([^ ]+) Stroke\("([^"]+)"\)"#).unwrap();
    }

    let groups = RE.captures(line)?;
    let time_str = groups.get(1).map(|m| m.as_str())?;
    let time = time_str.parse::<DateTime<Utc>>().ok()?;
    let stroke = groups.get(2).map(|m| m.as_str())?;

    Some((time, stroke.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse_lines(lines: &str) -> String {
        parse_raw(lines.lines().map(|l| l.to_string()).collect()).join("\n")
    }

    #[test]
    fn parse_simple() {
        let raw = r#"
2020-11-29T16:20:50.529-08:00 Stroke("EU") => [Replace(0, "I")]
2020-11-29T16:20:52.333-08:00 Stroke("SR-PBT") => [Replace(0, " haven\'t")
2020-11-29T16:20:53.507-08:00 Stroke("PHRAEUD") => [Replace(0, " played")]
        "#;
        let expected = r#"
    0,EU
 1804,SR-PBT
 1174,PHRAEUD"#;

        assert_eq!(String::from("\n") + &parse_lines(raw), expected);
    }

    #[test]
    fn parse_long_time() {
        let raw = r#"
2020-11-29T16:20:50.529-08:00 Stroke("EU") => [Replace(0, "I")]
2020-11-29T16:20:52.333-08:00 Stroke("SR-PBT") => [Replace(0, " haven\'t")
2020-12-30T16:20:53.507-08:00 Stroke("PHRAEUD") => [Replace(0, " played")]
        "#;
        let expected = r#"
    0,EU
 1804,SR-PBT
65535,PHRAEUD"#;

        assert_eq!(String::from("\n") + &parse_lines(raw), expected);
    }
}
