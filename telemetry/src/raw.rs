use crate::parsed::{Content, LogEntry};
use chrono::{DateTime, Utc};
use regex::Regex;
use std::{error::Error, fmt};

/// Parse a raw line from a log file into a common data format
pub fn parse_raw(raw: &str) -> Result<LogEntry, Box<dyn Error>> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"^([^ ]+) Stroke\("([^"]+)"\) => (.+)$"#).unwrap();
            // Regex::new(r#"^([^ ]+) Stroke\("([^"]+)"\) => \[Replace\((\d+), "(.*)"\)\]$"#).unwrap();
        static ref TEXT_RE: Regex =
            Regex::new(r#"^\[Replace\((\d+), "(.*)"\)\]$"#).unwrap();
    }

    let groups = RE.captures(raw).ok_or(ParseError::RegexDoesNotMatch)?;
    let time = groups
        .get(1)
        .map(|m| m.as_str())
        .ok_or(ParseError::NoTimeString)?;
    let time = time.parse::<DateTime<Utc>>()?;
    let time = time.timestamp_millis();
    let stroke = groups
        .get(2)
        .map(|m| m.as_str())
        .ok_or(ParseError::NoStroke)?;
    let payload = groups
        .get(3)
        .map(|m| m.as_str())
        .ok_or(ParseError::NoPayload)?;

    let content = if let Some(groups) = TEXT_RE.captures(payload) {
        let backspace_num = groups
            .get(1)
            .map(|m| m.as_str())
            .ok_or(ParseError::NoPayload)?;
        let backspace_num = backspace_num.parse::<u32>()?;
        let text = groups
            .get(2)
            .map(|m| m.as_str())
            .ok_or(ParseError::NoPayload)?;
        // unescape the quotes
        let text = text.replace(r#"\""#, r#"""#);
        let text = text.replace(r#"\'"#, r#"'"#);
        let text = text.to_string();

        Content::Replace {
            backspace_num,
            text,
        }
    } else if payload == "[NoOp]" {
        Content::NoOp
    } else {
        // anything besides text and noop is regarded as a command
        Content::Command
    };

    return Ok(LogEntry {
        time,
        stroke: stroke.to_string(),
        content,
    });
}

#[derive(Debug)]
enum ParseError {
    RegexDoesNotMatch,
    NoTimeString,
    NoStroke,
    NoPayload, // no Replace, Command, or NoOp
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_escaping() {
        assert_eq!(
            parse_raw(r#"2020-11-29T16:20:50.529-08:00 Stroke("EU") => [Replace(0, " haven\'t")]"#)
                .unwrap(),
            LogEntry {
                time: "2020-11-29T16:20:50.529-08:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .timestamp_millis(),
                stroke: "EU".to_string(),
                content: Content::Replace {
                    text: r#" haven't"#.to_string(),
                    backspace_num: 0,
                },
            }
        );
        assert_eq!(
            parse_raw(r#"2020-11-29T16:20:50.529-08:00 Stroke("KW-GS") => [Replace(0, " \"")]"#)
                .unwrap(),
            LogEntry {
                time: "2020-11-29T16:20:50.529-08:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .timestamp_millis(),
                stroke: "KW-GS".to_string(),
                content: Content::Replace {
                    text: r#" ""#.to_string(),
                    backspace_num: 0,
                },
            }
        );
    }

    #[test]
    fn parse_line_empty() {
        assert_eq!(
            parse_raw(r#"2020-11-29T16:20:50.529-08:00 Stroke("*") => [Replace(3, "")]"#).unwrap(),
            LogEntry {
                time: "2020-11-29T16:20:50.529-08:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .timestamp_millis(),
                stroke: "*".to_string(),
                content: Content::Replace {
                    text: r#""#.to_string(),
                    backspace_num: 3,
                },
            }
        );
        assert_eq!(
            parse_raw(r#"2020-11-29T16:20:50.529-08:00 Stroke("KPA") => [NoOp]"#).unwrap(),
            LogEntry {
                time: "2020-11-29T16:20:50.529-08:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .timestamp_millis(),
                stroke: "KPA".to_string(),
                content: Content::NoOp,
            }
        );
    }

    #[test]
    fn parse_lines_commands() {
        assert_eq!(
            parse_raw(r#"2020-12-01T21:26:55.194-08:00 Stroke("SRO*PL") => [Shell("osascript", ["-e", "set volume output volume (output volume of (get volume settings) - 5)"])]"#).unwrap(),
            LogEntry {
                time: "2020-12-01T21:26:55.194-08:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .timestamp_millis(),
                stroke: "SRO*PL".to_string(),
                content: Content::Command,
            }
        );
        assert_eq!(
            parse_raw(r#"2020-12-12T12:59:48.940-08:00 Stroke("PHR*UP") => [Replace(0, "ag -iQ \'\"\"\' --ignore dict_full.json ~/plojo/cli/runtime_files/"), Keys(Layout('a'), [Control]), Keys(Special(RightArrow), [Alt]), Keys(Special(RightArrow), [])]"#).unwrap(),
            LogEntry {
                time: "2020-12-12T12:59:48.940-08:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .timestamp_millis(),
                stroke: "PHR*UP".to_string(),
                content: Content::Command,
            }
        );
    }
}
