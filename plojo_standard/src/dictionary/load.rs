use crate::{Text, TextAction, Translation};
use plojo_core::{Command, Stroke};
use regex::Regex;
use serde_json::{self, Error as JsonError, Value};
use std::{error::Error, fmt};

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
/// The glue operator allows text to be attached (space suppressed) to other glued strokes.
/// - `{&a}`, `{&b}`, `{&c}`, etc. make up the fingerspelling dictionary
/// - `{&th}`: multi letter text is allowed as well
///
/// Number strokes (strokes that use the number bar containing only numbers, and are not in the
/// dictionary) are glued by default
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
/// - `{~|text}` or `{^~|text^}` where the attach operator is optional and the text can be changed
///     - Note that currently this operator can be recognized, but does nothing
///
/// ### Punctuation symbols
/// - `{.}`, `{?}`, `{!}`: inserts a the punctuation joined to the previous word and uppercases anything next
/// - `{,}`, `{:}`, `{;}`: inserts the punctuation joined to the previous word
///
/// ### Retrospective Space
/// - `{*?}`: retrospectivly add space before the previous translated word
/// - `{*!}`: retrospectivly remove space before the previous translated word
///
/// ### Literal symbols
/// - `{bracketleft}`: inserts a literal opening bracket (`{`)
/// - `{bracketright}`: inserts a literal closing bracket (`}`)
///
///
/// ## Differences from plover
///
/// - The empty text commmand (`{}`) does not do anything. In plover, this stroke cancels the
///   formatting of the next word.
/// - Retrospective space adding/removing works on the previous word, not the previous stroke
pub(super) fn load_dicts(contents: &str) -> Result<Entries, ParseError> {
    let value: Value = serde_json::from_str(&contents)?;

    let object_entries = value.as_object().ok_or(ParseError::NotEntries)?;

    let mut result_entries = vec![];

    for (stroke, translation) in object_entries {
        let stroke = parse_stroke(stroke)?;
        match translation {
            Value::String(translation_str) => {
                let parsed = parse_translation(translation_str)?;
                result_entries.push((stroke, parsed));
            }
            Value::Object(obj) => {
                let commands = obj.get("cmds").ok_or(ParseError::InvalidTranslation(
                    "cmds key not found".to_string(),
                ))?;
                let parsed: Vec<Command> = serde_json::from_value(commands.clone())?;
                let mut text_actions: Option<Vec<TextAction>> = None;
                if let Some(raw_actions) = obj.get("text_actions") {
                    text_actions = Some(serde_json::from_value(raw_actions.clone())?);
                }
                result_entries.push((
                    stroke,
                    vec![Translation::Command {
                        cmds: parsed,
                        text_actions,
                    }],
                ));
            }
            _ => {
                return Err(ParseError::UnknownTranslation(translation.to_string()));
            }
        }
    }

    Ok(result_entries)
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    // if the JSON file does not exclusively contain an object with entries
    NotEntries,
    InvalidStroke(String),
    UnknownTranslation(String),
    EmptyTranslation,
    InvalidTranslation(String),
    // a special action is one that is wrapped in brackets in the translation
    InvalidSpecialAction(String),
    JsonError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseError {}

impl From<JsonError> for ParseError {
    fn from(e: JsonError) -> Self {
        ParseError::JsonError(e.to_string())
    }
}

type Entries = Vec<(Stroke, Vec<Translation>)>;

fn parse_stroke(s: &str) -> Result<Stroke, ParseError> {
    let stroke = Stroke::new(s);
    if stroke.is_valid() {
        Ok(stroke)
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
    let mut in_brackets = false;
    // using char_indices here to handle utf-8 chars, which might not be 1 byte long
    for (end, c) in t.char_indices() {
        // pass anything in brackets to parse_special and everything else to parse_as_text
        match c {
            '{' => {
                if start < end {
                    // if there's anything before the bracket, that should be a text literal
                    translations.push(parse_as_text(&t[start..end]));
                }
                // adding 1 here is fine because '{' is one byte long
                start = end + 1;
                in_brackets = true;
            }
            '}' => {
                if !in_brackets {
                    return Err(ParseError::InvalidTranslation(
                        "Unbalanced brackets: extra closing bracket(s)".to_string(),
                    ));
                }

                translations.append(&mut parse_special(&t[start..end])?);
                // adding 1 here is fine because '{' is one byte long
                start = end + 1;
                in_brackets = false;
            }
            // ignore everything else
            _ => {}
        }
    }

    if in_brackets {
        return Err(ParseError::InvalidTranslation(
            "Unbalanced brackets: extra opening bracket(s)".to_string(),
        ));
    } else if start < t.len() {
        // if there's still more text, add that as well as a text literal
        translations.push(parse_as_text(&t[start..]));
    }

    Ok(translations)
}

lazy_static! {
    // 1st capturing group: possible caret (^)
    // 2nd capturing group: possible text to apply orthography to
    // 3rd capturing group: possible caret (^)
    static ref ATTACHED_REGEX: Regex = Regex::new(r"^(\^?)([^\^]*)(\^?)$").unwrap();
    // part of the attached_regex (which checks for attach operator)
    // checks if the content of the suffix starts with `~|`, to carry the capitalization
    static ref CARRYING_CAP: Regex = Regex::new(r"^~\|(.+)$").unwrap();
}

/// Parses "special actions" which are in the translation surrounded by brackets
fn parse_special(t: &str) -> Result<Vec<Translation>, ParseError> {
    match t {
        // do nothing for empty action
        "" => Ok(vec![]),
        // period
        "." => Ok(vec![
            Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            Translation::Text(Text::Lit(".".to_string())),
            Translation::Text(Text::TextAction(vec![TextAction::case(true, true)])),
        ]),
        // question mark
        "?" => Ok(vec![
            Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            Translation::Text(Text::Lit("?".to_string())),
            Translation::Text(Text::TextAction(vec![TextAction::case(true, true)])),
        ]),
        // exclamation mark
        "!" => Ok(vec![
            Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            Translation::Text(Text::Lit("!".to_string())),
            Translation::Text(Text::TextAction(vec![TextAction::case(true, true)])),
        ]),
        // comma
        "," => Ok(vec![
            Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            Translation::Text(Text::Lit(",".to_string())),
        ]),
        // colon
        ":" => Ok(vec![
            Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            Translation::Text(Text::Lit(":".to_string())),
        ]),
        // colon
        ";" => Ok(vec![
            Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            Translation::Text(Text::Lit(";".to_string())),
        ]),
        // capitalize next word
        "-|" => Ok(vec![Translation::Text(Text::TextAction(vec![
            TextAction::case(true, true),
        ]))]),
        // capitalize previous word
        "*-|" => Ok(vec![Translation::Text(Text::TextAction(vec![
            TextAction::case(false, true),
        ]))]),
        // add space to prev word
        "*?" => Ok(vec![Translation::Text(Text::TextAction(vec![
            TextAction::space(false, true),
        ]))]),
        // remove space from prev word
        "*!" => Ok(vec![Translation::Text(Text::TextAction(vec![
            TextAction::space(false, false),
        ]))]),
        // insert literal bracket
        "bracketleft" => Ok(vec![Translation::Text(Text::Lit("{".to_string()))]),
        "bracketright" => Ok(vec![Translation::Text(Text::Lit("}".to_string()))]),
        _t => {
            // check for prefix/suffix action (attach operator)
            let matched = ATTACHED_REGEX.captures(_t);
            if let Some(groups) = matched {
                // all regexes have 1 as the first capturing group
                // a caret in front means its either a suppress space or apply orthography
                if &groups[1] == "^" {
                    // nothing in the text section, just a simple suppress space stroke
                    if groups[2].len() == 0 {
                        return Ok(vec![
                            Translation::Text(Text::TextAction(vec![TextAction::space(
                                true, false,
                            )])),
                            Translation::Text(Text::Lit("".to_string())),
                            Translation::Text(Text::TextAction(vec![TextAction::space(
                                true, false,
                            )])),
                        ]);
                    }

                    // simply ignore the `~|` in carrying capitalization for now
                    let mut content = groups[2].to_string();
                    if let Some(carrying_cap) = CARRYING_CAP.captures(&content) {
                        content = carrying_cap[1].to_string();
                    }

                    // apply orthography with an attached action
                    if &groups[3] == "^" {
                        // suppress next space if needed
                        return Ok(vec![
                            Translation::Text(Text::Attached(content)),
                            Translation::Text(Text::TextAction(vec![TextAction::space(
                                true, false,
                            )])),
                        ]);
                    } else {
                        return Ok(vec![Translation::Text(Text::Attached(content))]);
                    }
                } else if &groups[3] == "^" {
                    // simply ignore the `~|` in carrying capitalization for now
                    let mut content = groups[2].to_string();
                    if let Some(carrying_cap) = CARRYING_CAP.captures(&content) {
                        content = carrying_cap[1].to_string();
                    }

                    // caret at end is a prefix stroke
                    return Ok(vec![
                        Translation::Text(Text::Lit(content)),
                        Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                    ]);
                }
                // no caret, ignore it

                // simply ignore the `~|` in carrying capitalization for now
                let content = groups[2].to_string();
                if let Some(carrying_cap) = CARRYING_CAP.captures(&content) {
                    let content = carrying_cap[1].to_string();

                    return Ok(vec![Translation::Text(Text::Lit(content))]);
                }
            }

            // check for glued operator
            if _t.len() >= 2 && _t.get(0..1) == Some(&"&") {
                if let Some(text) = _t.get(1..) {
                    return Ok(vec![Translation::Text(Text::Glued(text.to_string()))]);
                }
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
    use plojo_core::{Key, Modifier, SpecialKey};
    use std::collections::HashSet;
    use std::iter::FromIterator;

    type Entry = (Stroke, Vec<Translation>);

    #[test]
    fn test_basic_parse_dictionary() {
        let contents = r#"
{
"TP": "if",
"KPA": "{}{-|}",
"-T/WUPB": "The One"
}
        "#;
        let parsed = load_dicts(contents).unwrap();
        let parsed: HashSet<Entry> = HashSet::from_iter(parsed.iter().cloned());

        let expect = vec![
            (
                Stroke::new("TP"),
                vec![Translation::Text(Text::Lit("if".to_string()))],
            ),
            (
                Stroke::new("KPA"),
                vec![Translation::Text(Text::TextAction(vec![TextAction::case(
                    true, true,
                )]))],
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
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
                Translation::Text(Text::Lit("".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
            ]
        );
        // `{^^}` should also suppress space
        assert_eq!(
            parse_translation("{^^}").unwrap(),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
                Translation::Text(Text::Lit("".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
            ]
        );
        // `{^}sh` should simply join "sh" to the previous word
        assert_eq!(
            parse_translation("{^}sh").unwrap(),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
                Translation::Text(Text::Lit("".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
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

    #[test]
    fn test_translation_text_actions() {
        // uppercase next word
        assert_eq!(
            parse_translation("{-|}").unwrap(),
            vec![Translation::Text(Text::TextAction(vec![TextAction::case(
                true, true
            )])),]
        );
        // uppercase next word and suppress space
        assert_eq!(
            parse_translation("{^}{-|}").unwrap(),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
                Translation::Text(Text::Lit("".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false),])),
                Translation::Text(Text::TextAction(vec![TextAction::case(true, true)])),
            ]
        );
        // uppercase previus word
        assert_eq!(
            parse_translation("{*-|}").unwrap(),
            vec![Translation::Text(Text::TextAction(vec![TextAction::case(
                false, true
            )])),]
        );
        // add space to prev word
        assert_eq!(
            parse_translation("{*?}").unwrap(),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(false, true)
            ])),]
        );
        // remove space from prev word
        assert_eq!(
            parse_translation("{*!}").unwrap(),
            vec![Translation::Text(Text::TextAction(vec![
                TextAction::space(false, false)
            ])),]
        );
    }

    #[test]
    fn test_translation_literal_bracket() {
        assert_eq!(
            parse_translation("{bracketleft}").unwrap(),
            vec![Translation::Text(Text::Lit("{".to_string())),]
        );
        assert_eq!(
            parse_translation("{bracketright}").unwrap(),
            vec![Translation::Text(Text::Lit("}".to_string())),]
        );
    }

    // only testing parsing for now
    #[test]
    fn test_translation_carrying_capitalization() {
        // quote attached to next word
        assert_eq!(
            parse_translation(r#"{~|"^}"#).unwrap(),
            vec![
                Translation::Text(Text::Lit("\"".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            ]
        );
        // parentheses attached to previous word
        assert_eq!(
            parse_translation(r#"{^~|(}"#).unwrap(),
            vec![Translation::Text(Text::Attached("(".to_string())),]
        );
        // quote attached on both sides
        assert_eq!(
            parse_translation(r#"{^~|'^}"#).unwrap(),
            vec![
                Translation::Text(Text::Attached("'".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            ]
        );
        // quote followed by word
        assert_eq!(
            parse_translation(r#"{~|'^}cause"#).unwrap(),
            vec![
                Translation::Text(Text::Lit("'".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit("cause".to_string())),
            ]
        );
        // long text carrying capitalized
        assert_eq!(
            parse_translation(r#"{~|hello^}"#).unwrap(),
            vec![
                Translation::Text(Text::Lit("hello".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
            ]
        );
    }

    #[test]
    fn test_translation_punctuation() {
        assert_eq!(
            parse_translation("{.}").unwrap(),
            vec![
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit(".".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::case(true, true)])),
            ]
        );
    }

    #[test]
    fn test_translation_unicode() {
        assert_eq!(
            parse_translation("©").unwrap(),
            vec![Translation::Text(Text::Lit("©".to_string()))]
        );
    }

    #[test]
    fn test_translation_empty_err() {
        assert_eq!(
            parse_translation("").unwrap_err(),
            ParseError::EmptyTranslation
        );
    }

    #[test]
    fn test_commands_parse_dictionary() {
        let contents = r#"
{
"UP": {"cmds": [{ "Keys": [{"Special": "UpArrow"}, []] }]},
"TEGT": {"cmds": [{ "Keys": [{"Layout": "a"}, ["Meta"]] }]}
}
        "#;
        let parsed = load_dicts(contents).unwrap();
        let parsed: HashSet<Entry> = HashSet::from_iter(parsed.iter().cloned());

        let expect = vec![
            (
                Stroke::new("UP"),
                vec![Translation::Command {
                    cmds: vec![Command::Keys(Key::Special(SpecialKey::UpArrow), vec![])],
                    text_actions: None,
                }],
            ),
            (
                Stroke::new("TEGT"),
                vec![Translation::Command {
                    cmds: vec![Command::Keys(Key::Layout('a'), vec![Modifier::Meta])],
                    text_actions: None,
                }],
            ),
        ];
        let expect: HashSet<Entry> = HashSet::from_iter(expect.iter().cloned());

        assert_eq!(parsed, expect);
    }
}
