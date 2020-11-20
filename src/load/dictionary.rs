use crate::stroke;
use crate::translator::dictionary::Dictionary;
use crate::{Stroke, Text, TextAction, Translation};
use regex::Regex;
use serde_json;
use std::fs;
use std::iter::FromIterator;

/// Loads the dictionary
///
/// # File format
/// The dictionary file format is similar to the Plover dictionary. Currently, to be compatible with
/// Plover, all dictionary entries must be in the form of a key and value in a single JSON file. The
/// key should be a valid stroke or series of strokes joined by `/`. The value can consist of
/// literal text with any formatting actions or commands (known as "special actions"), which are
/// surrounded by brackets (`{like this}`).
///
/// ## Formatting actions
///
/// ### Infix and suffixes
/// - `{^}` is the attach operator (same as suppress space, which can also be written as `{^^}`)
/// - `{^ish}` is an orthographic-aware suffix that will add "ish" to the end of the previous word.
///     - E.g. `RED/EURB` will output reddish. Note that a second "d" due to the orthography rules
/// - `{^}ish` is a suffix with the text outside the operator—this means that the text will simply
///   be attached (space is suppressed) without grammar rules. Using this stroke in the previous
///   example would give instead redish.
/// - `{^-to-^}` is an infix, e.g. day-to-day. Note that this is the same as `{^}-to-{^}`
/// - `{in^}` is a prefix, e.g. influx. Note that this is the same as `in{^}` as there are no
///   orthography rules for the beginning of words.
///
/// Overall, `{^}` will only suppress space unless there is additional text inside the brackets
/// after the caret sign, in which case it will apply orthography rules
///
/// ### Glue operator
/// Not yet implemented
///
/// ### Capitalizing
/// The first letter of the next (or previous) translation can be capitalized
///
/// - `{-|}`: capitalize next word (`{^}{-|}` also suppresses space)
/// - `{*-|}`: capitalize previous word (`{^}{*-|}` also suppresses space)
///     - this can be used in conjunction with suffixes: `{*-|}{^ville}` will capitalize the
///       previous word and add `ville` to the end. For example: `cat` would become `Catville`.
///
/// ### Carrying capitalizing
/// Not yet implemented
pub fn load(filename: &str) -> Result<Dictionary, ParseError> {
    // TODO: handle IO error properly
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    parse_dictionary(&contents).map(Dictionary::from_iter)
}

#[derive(Debug)]
pub enum ParseError {
    // if the JSON file does not exclusively contain an object with entries
    NotEntries,
    InvalidStroke(String),
    // currently, a translation must be a string
    NonStringTranslation(String),
    EmptyTranslation,
    InvalidTranslation(String),
    // a special action is one that is wrapped in brackets in the translation
    InvalidSpecialAction(String),
    JsonError(String),
}

impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        ParseError::JsonError(e.to_string())
    }
}

type Entries = Vec<(Stroke, Vec<Translation>)>;

/// Parses a dictionary JSON file into a list of the stroke and translation entries
fn parse_dictionary(contents: &str) -> Result<Entries, ParseError> {
    let value: serde_json::Value = serde_json::from_str(&contents)?;

    let object_entries = value.as_object().ok_or(ParseError::NotEntries)?;

    let mut result_entries = vec![];

    for (stroke, translation) in object_entries {
        let stroke = parse_stroke(stroke)?;
        let translation_str = translation
            .as_str()
            .ok_or(ParseError::NonStringTranslation(translation.to_string()))?;
        let parsed = parse_translation(translation_str)?;
        result_entries.push((stroke, parsed));
    }

    Ok(result_entries)
}

fn parse_stroke(s: &str) -> Result<Stroke, ParseError> {
    if stroke::is_valid_stroke(s) {
        Ok(Stroke::new(s))
    } else {
        Err(ParseError::InvalidStroke(s.to_string()))
    }
}

fn parse_translation(t: &str) -> Result<Vec<Translation>, ParseError> {
    if t.len() == 0 {
        return Err(ParseError::EmptyTranslation);
    }

    let mut translations = vec![];
    let mut start = 0;
    let mut end = 0;
    let mut in_brackets = false;
    // pass anything in brackets to parse_special and everything else to parse_as_text
    for c in t.chars() {
        match c {
            '{' => {
                if start < end {
                    // if there's anything before, that should be a text literal
                    translations.push(parse_as_text(&t[start..end]));
                }
                end += 1;
                start = end;
                in_brackets = true;
            }
            '}' => {
                if !in_brackets {
                    return Err(ParseError::InvalidTranslation(
                        "Unbalanced brackets: extra closing bracket(s)".to_string(),
                    ));
                }

                translations.append(&mut parse_special(&t[start..end])?);
                end += 1;
                start = end;
                in_brackets = false;
            }
            _ => {
                end += 1;
            }
        }
    }

    if in_brackets {
        return Err(ParseError::InvalidTranslation(
            "Unbalanced brackets: extra opening bracket(s)".to_string(),
        ));
    } else if start < end {
        // if there's anything before, that should be a text literal
        translations.push(parse_as_text(&t[start..end]));
    }

    Ok(translations)
}

lazy_static! {
    // 1st capturing group: possible caret (^)
    // 2nd capturing group: possible text to apply orthography to
    // 3rd capturing group: possible caret (^)
    static ref SUFFIX_REGEX: Regex = Regex::new(r"^(\^?)([^\^]*)(\^?)$").unwrap();
}

/// Parses "special actions" which are in the translation surrounded by brackets
fn parse_special(t: &str) -> Result<Vec<Translation>, ParseError> {
    match t {
        "-|" => Ok(vec![Translation::Text(Text::TextAction(vec![
            TextAction::case(true, true),
        ]))]),
        "*-|" => Ok(vec![Translation::Text(Text::TextAction(vec![
            TextAction::case(true, true),
            TextAction::space(true, false),
        ]))]),
        _t => {
            // check for prefix/suffix action
            let matched = SUFFIX_REGEX.captures(_t);
            if let Some(groups) = matched {
                // all regexes have 1 as the first capturing group
                // a caret in front means its either a suppress space or apply orthography
                if &groups[1] == "^" {
                    // nothing in the text section, just a simple suppress space stroke
                    if groups[2].len() == 0 {
                        return Ok(vec![Translation::Text(Text::TextAction(vec![
                            TextAction::space(true, false),
                        ]))]);
                    }

                    // apply orthography with an attached action
                    if &groups[3] == "^" {
                        // suppress next space if needed
                        return Ok(vec![
                            Translation::Text(Text::Attached(groups[2].to_string())),
                            Translation::Text(Text::TextAction(vec![TextAction::space(
                                true, false,
                            )])),
                        ]);
                    } else {
                        return Ok(vec![Translation::Text(Text::Attached(
                            groups[2].to_string(),
                        ))]);
                    }
                } else if &groups[3] == "^" {
                    // caret at end is a prefix stroke
                    return Ok(vec![
                        Translation::Text(Text::Lit(groups[2].to_string())),
                        Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                    ]);
                }
                // no caret, ignore it
            }

            return Err(ParseError::InvalidSpecialAction(_t.to_string()));
        }
    }
}

// Parses directly as a text literal
fn parse_as_text(t: &str) -> Translation {
    Translation::Text(Text::Lit(t.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    type Entry = (Stroke, Vec<Translation>);

    #[test]
    fn test_basic_parse_dictionary() {
        let contents = r#"
{
"TP": "if",
"-T/WUPB": "The One"
}
        "#;
        let parsed = parse_dictionary(contents).unwrap();
        let parsed: HashSet<Entry> = HashSet::from_iter(parsed.iter().cloned());

        let expect = vec![
            (
                Stroke::new("TP"),
                vec![Translation::Text(Text::Lit("if".to_string()))],
            ),
            (
                Stroke::new("-T/WUPB"),
                vec![Translation::Text(Text::Lit("The One".to_string()))],
            ),
        ];
        let expect: HashSet<Entry> = HashSet::from_iter(expect.iter().cloned());

        assert_eq!(parsed, expect);
    }

    #[test]
    fn test_translation_suffix() {
        // `{^}` should suppress space
        assert_eq!(
            parse_translation("{^}").unwrap(),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(true, false)
            ]))]
        );
        // `{^^}` should also suppress space
        assert_eq!(
            parse_translation("{^^}").unwrap(),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(true, false)
            ]))]
        );
        // `{^}sh` should simply join "sh" to the previous word
        assert_eq!(
            parse_translation("{^}sh").unwrap(),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit("sh".to_string()))
            ]
        );
        // `{^ish}` should be an attached (apply orthography) ish
        assert_eq!(
            parse_translation("{^ish}").unwrap(),
            vec![Translation::Text(Text::Attached("ish".to_string())),]
        );
        // `{^-to-^}` should be "-to-" attached with orthography with space suppressed following it
        assert_eq!(
            parse_translation("{^-to-^}").unwrap(),
            vec![
                Translation::Text(Text::Attached("-to-".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            ]
        );
        // `{in^}` should be an "in" followed by a suppressed space
        assert_eq!(
            parse_translation("{in^}").unwrap(),
            vec![
                Translation::Text(Text::Lit("in".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            ]
        );
    }
}
