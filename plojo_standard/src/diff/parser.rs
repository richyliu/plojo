use crate::{Text, TextActionType};
use orthography::apply_orthography;
use regex::Regex;
use std::collections::HashMap;
use std::iter::FromIterator;

mod orthography;

lazy_static! {
    // whether a translation contains only digits or the center dash
    // although the regex will mark "-" as a number, such a stroke is not possible
    static ref NUMBER_TRANSLATION_REGEX: Regex = Regex::new(r"^[0-9\-]+$").unwrap();
    // whether a translation contains only digits, in which case it will be glued
    static ref NUMBERS_ONLY_REGEX: Regex = Regex::new(r"^[0-9]+$").unwrap();
}

/// For the translation_to_string function
#[derive(Debug, PartialEq)]
enum TextInternal {
    // text literal, such as from a translation or joining attached words via orthography rules
    Lit(String),
    // glued strokes are "glued" to each other
    Glued(String),
    // string representation of an unknown stroke; upper/lowercase actions do not apply to them
    Unknown(String),
    // actions that could apply to the previous or the next word and its boolean value (upper/lower, no/space)
    Actions(HashMap<TextActionType, bool>),
}

/// Converts translations into their string representation by adding spaces in between words and
/// applying text actions.
pub(super) fn parse_translation(translations: Vec<Text>) -> String {
    let mut final_string = String::from("");
    let mut merged: Vec<_> = merge_translations(translations)
        .into_iter()
        .map(Some)
        .collect();
    // push another None so that all the values have a chance to be `cur`
    merged.push(None);
    merged.push(None);

    // value of the current
    let mut prev: Option<TextInternal> = None;
    let mut cur: Option<TextInternal> = None;
    let mut next: Option<TextInternal> = None;
    for text in merged {
        // no need for mut or an initial value because it's assigned in the match statement below
        let str;
        let mut force_space = true;
        let mut uppercase: Option<bool> = None;

        // suppress space if previous stroke was a glued and this stroke is glued too
        if let Some(TextInternal::Glued(_)) = &prev {
            if let Some(TextInternal::Glued(_)) = &cur {
                force_space = false;
            }
        }

        // looking backwards first for text actions
        if let Some(TextInternal::Actions(actions)) = &prev {
            if let Some(is_force_space) = actions.get(&TextActionType::SpaceNext) {
                force_space = *is_force_space;
            }
            if let Some(is_uppercase) = actions.get(&TextActionType::CaseNext) {
                uppercase = Some(*is_uppercase);
            }
        }

        // then look ahead, because things ahead can override things behind
        if let Some(TextInternal::Actions(actions)) = &next {
            if let Some(is_force_space) = actions.get(&TextActionType::SpacePrev) {
                force_space = *is_force_space;
            }
            if let Some(is_uppercase) = actions.get(&TextActionType::CasePrev) {
                uppercase = Some(*is_uppercase);
            }
        }

        match &cur {
            Some(TextInternal::Lit(lit)) => {
                str = lit;
            }
            Some(TextInternal::Glued(lit)) => {
                str = lit;
            }
            Some(TextInternal::Unknown(unknown)) => {
                // cannot force upper/lowercase for an unknown stroke
                uppercase = None;
                str = unknown;
            }
            _ => {
                // do nothing for actions or anything else and advance to next item in vec
                prev = cur;
                cur = next;
                next = text;
                continue;
            }
        }

        // add the text to the final string
        if force_space {
            final_string.push(' ');
        }
        if let Some(uppercase) = uppercase {
            final_string.push_str(&word_change_first_letter(str.clone(), uppercase));
        } else {
            final_string.push_str(&str);
        }

        // advance to next item in vec
        prev = cur;
        cur = next;
        next = text;
    }

    final_string
}

/// Simplifies a series of translations by merging consecutive text actions into a hashset
fn merge_translations(translations: Vec<Text>) -> Vec<TextInternal> {
    // merging 0 translations results in 0 text internals
    if translations.len() == 0 {
        return vec![];
    }

    // accumulated text literals and text actions
    let mut acc: Vec<TextInternal> = Vec::with_capacity(translations.len());
    // consecutive actions are added to this
    let mut actions: Option<HashMap<TextActionType, bool>> = None;
    // consecutive attached words after a word are added to this
    let mut words: Option<Vec<String>> = None;
    // whether the first text in words was an attached (in which case to suppress space)
    let mut first_word_attached: bool = false;

    // merge all consecutive text actions into a set
    for cur in translations {
        // check for attached text first
        if let Text::Attached(attached) = cur {
            // push any text actions first
            if let Some(text_actions) = actions {
                acc.push(TextInternal::Actions(text_actions));
                actions = None;
            }

            if let Some(mut prev_words) = words {
                prev_words.push(attached);
                words = Some(prev_words);
            } else {
                words = Some(vec![attached]);
                first_word_attached = true;
            }
        } else {
            // not an attached text, so if there are attached words, apply orthography and push to acc
            if let Some(attached) = words {
                // suppress space also if the first in words was an attached
                if first_word_attached {
                    let prev = acc.pop();
                    if let Some(prev) = prev {
                        // add the suppress space to previous actions if there are any
                        if let TextInternal::Actions(mut prev_actions) = prev {
                            prev_actions.insert(TextActionType::SpaceNext, false);
                            acc.push(TextInternal::Actions(prev_actions));
                        } else {
                            // previous was not an action
                            acc.push(prev);
                            acc.push(TextInternal::Actions(HashMap::from_iter(vec![(
                                TextActionType::SpaceNext,
                                false,
                            )])));
                        }
                    } else {
                        acc.push(TextInternal::Actions(HashMap::from_iter(vec![(
                            TextActionType::SpaceNext,
                            false,
                        )])));
                    }
                }
                acc.push(TextInternal::Lit(apply_orthography(attached)));
                words = None;
            }

            match cur {
                Text::Lit(text) => {
                    // push any text actions first
                    if let Some(text_actions) = actions {
                        acc.push(TextInternal::Actions(text_actions));
                        actions = None;
                    }

                    // for number-only translations, push is as glued
                    if NUMBERS_ONLY_REGEX.is_match(&text) {
                        acc.push(TextInternal::Glued(text));
                    } else {
                        words = Some(vec![text]);
                    }
                    first_word_attached = false;
                }
                Text::UnknownStroke(stroke) => {
                    // push any text actions first
                    if let Some(text_actions) = actions {
                        acc.push(TextInternal::Actions(text_actions));
                        actions = None;
                    }

                    acc.push(TextInternal::Unknown(stroke.to_raw()));
                }
                Text::TextAction(text_actions) => {
                    match actions {
                        Some(mut prev_actions) => {
                            for a in text_actions {
                                prev_actions.insert(a.action_type, a.val);
                            }
                            actions = Some(prev_actions);
                        }
                        None => {
                            let mut new_actions = HashMap::new();
                            for a in text_actions {
                                new_actions.insert(a.action_type, a.val);
                            }
                            actions = Some(new_actions);
                        }
                    };
                }
                Text::Attached(_) => {
                    // already handled above; shouldn't be here
                    panic!("this shouldn't be possible");
                }
                Text::Glued(text) => {
                    // push any text actions first
                    if let Some(text_actions) = actions {
                        acc.push(TextInternal::Actions(text_actions));
                        actions = None;
                    }

                    acc.push(TextInternal::Glued(text));
                    first_word_attached = false;
                }
            };
        }
    }

    // push remaining actions
    if let Some(a) = actions {
        acc.push(TextInternal::Actions(a));
    }

    // apply orthography and push remaining words
    if let Some(words) = words {
        // suppress space also if the first in words was an attached
        if first_word_attached {
            let prev = acc.pop();
            if let Some(prev) = prev {
                // add the suppress space to previous actions if there are any
                if let TextInternal::Actions(mut prev_actions) = prev {
                    prev_actions.insert(TextActionType::SpaceNext, false);
                    acc.push(TextInternal::Actions(prev_actions));
                } else {
                    // previous was not an action
                    acc.push(prev);
                    acc.push(TextInternal::Actions(HashMap::from_iter(vec![(
                        TextActionType::SpaceNext,
                        false,
                    )])));
                }
            } else {
                acc.push(TextInternal::Actions(HashMap::from_iter(vec![(
                    TextActionType::SpaceNext,
                    false,
                )])));
            }
        }

        acc.push(TextInternal::Lit(apply_orthography(words)));
    }

    acc = acc
        .into_iter()
        .map(|t| {
            if let TextInternal::Unknown(ref num) = t {
                // turn a number ("unknown") into glued if it contains only digits
                if NUMBER_TRANSLATION_REGEX.is_match(num) {
                    // remove center dash
                    let without_dash = num.replace("-", "");
                    return TextInternal::Glued(without_dash);
                }
            }

            t
        })
        .collect();

    acc
}

fn word_change_first_letter(word: String, uppercase: bool) -> String {
    if let Some(first_letter) = word.get(0..1) {
        // capitalize or lowercase the first letter
        let result = if uppercase {
            first_letter.to_uppercase()
        } else {
            first_letter.to_lowercase()
        };

        let mut s = result.to_string();
        // add the rest of the word
        if let Some(rest) = word.get(1..) {
            s.push_str(rest);
        }

        s
    } else {
        // do nothing on empty word
        word
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TextAction;
    use plojo_core::Stroke;

    #[test]
    fn test_parse_empty() {
        let translated = parse_translation(vec![]);

        assert_eq!(translated, "");
    }

    #[test]
    fn test_parse_basic() {
        let translated = parse_translation(vec![
            Text::Lit("hello".to_string()),
            Text::Lit("hi".to_string()),
        ]);

        assert_eq!(translated, " hello hi");
    }

    #[test]
    fn test_parse_text_actions() {
        let translated = parse_translation(vec![
            Text::TextAction(vec![
                TextAction::space(true, false),
                TextAction::case(true, true),
            ]),
            Text::Lit("hello".to_string()),
            Text::Lit("hi".to_string()),
            Text::TextAction(vec![TextAction::case(true, true)]),
            Text::Lit("FOo".to_string()),
            Text::Lit("bar".to_string()),
            Text::Lit("baZ".to_string()),
            Text::TextAction(vec![TextAction::case(true, false)]),
            Text::TextAction(vec![TextAction::space(true, false)]),
            Text::Lit("NICE".to_string()),
            Text::TextAction(vec![TextAction::space(true, false)]),
            Text::Lit("".to_string()),
            Text::Lit("well done".to_string()),
        ]);

        assert_eq!(translated, "Hello hi FOo bar baZnICE well done");
    }

    #[test]
    fn test_parse_prev_word_text_actions() {
        let translated = parse_translation(vec![
            Text::Lit("hi".to_string()),
            Text::TextAction(vec![
                TextAction::case(false, true),
                TextAction::case(false, false),
            ]),
            Text::TextAction(vec![TextAction::case(true, false)]),
            Text::TextAction(vec![TextAction::case(false, true)]),
            Text::Lit("FOo".to_string()),
            Text::Lit("bar".to_string()),
            Text::TextAction(vec![
                TextAction::space(false, false),
                TextAction::case(false, true),
            ]),
            Text::Lit("hello".to_string()),
            Text::Lit("Hi a".to_string()),
            Text::TextAction(vec![
                TextAction::space(true, true),
                TextAction::case(true, false),
            ]),
            Text::TextAction(vec![TextAction::case(false, true)]),
            Text::TextAction(vec![
                TextAction::case(true, true),
                TextAction::case(true, true),
            ]),
            Text::Lit("nice".to_string()),
            Text::UnknownStroke(Stroke::new("TP-TDZ")),
            Text::TextAction(vec![
                TextAction::case(false, false),
                TextAction::space(false, false),
            ]),
            Text::Lit("nice".to_string()),
            Text::TextAction(vec![TextAction::space(true, false)]),
            Text::Lit("another".to_string()),
            Text::TextAction(vec![TextAction::space(false, true)]),
        ]);

        assert_eq!(translated, " Hi fOoBar hello Hi a NiceTP-TDZ nice another");
    }

    #[test]
    fn test_parse_line_start() {
        let translated = parse_translation(vec![
            Text::TextAction(vec![
                TextAction::space(true, false),
                TextAction::case(true, true),
            ]),
            Text::Lit("hello".to_string()),
            Text::Lit("hi".to_string()),
        ]);

        assert_eq!(translated, "Hello hi");
    }

    #[test]
    fn test_parse_glued() {
        let translated = parse_translation(vec![
            Text::Lit("hello".to_string()),
            Text::Glued("hi".to_string()),
            Text::Glued("hi".to_string()),
            Text::Lit("foo".to_string()),
            Text::Glued("two".to_string()),
            Text::Glued("three".to_string()),
            Text::TextAction(vec![TextAction::space(false, true)]),
        ]);

        assert_eq!(translated, " hello hihi foo two three");
    }

    #[test]
    fn test_merge_translations() {
        let translated = merge_translations(vec![
            Text::TextAction(vec![
                TextAction::space(true, true),
                TextAction::case(true, true),
            ]),
            Text::Lit("hello".to_string()),
            Text::Lit("hi".to_string()),
            Text::TextAction(vec![TextAction::case(true, false)]),
            Text::TextAction(vec![
                TextAction::case(false, true),
                TextAction::case(false, true),
                TextAction::case(true, false),
            ]),
            Text::TextAction(vec![
                TextAction::case(false, true),
                TextAction::space(false, false),
                TextAction::case(true, false),
            ]),
            Text::Lit("FOo".to_string()),
            Text::TextAction(vec![
                TextAction::case(false, true),
                TextAction::case(false, false),
            ]),
            Text::TextAction(vec![TextAction::case(false, false)]),
            Text::Lit("FOo".to_string()),
            Text::TextAction(vec![
                TextAction::case(true, false),
                TextAction::case(true, true),
            ]),
        ]);

        assert_eq!(
            translated,
            vec![
                TextInternal::Actions(HashMap::from_iter(vec![
                    (TextActionType::SpaceNext, true),
                    (TextActionType::CaseNext, true),
                ])),
                TextInternal::Lit("hello".to_string()),
                TextInternal::Lit("hi".to_string()),
                TextInternal::Actions(HashMap::from_iter(vec![
                    (TextActionType::CasePrev, true),
                    (TextActionType::SpacePrev, false),
                    (TextActionType::CaseNext, false),
                ])),
                TextInternal::Lit("FOo".to_string()),
                TextInternal::Actions(HashMap::from_iter(vec![(TextActionType::CasePrev, false),])),
                TextInternal::Lit("FOo".to_string()),
                TextInternal::Actions(HashMap::from_iter(vec![(TextActionType::CaseNext, true),])),
            ]
        );
    }

    #[test]
    fn test_merge_apply_orthography() {
        let translated = merge_translations(vec![
            Text::Lit("fancy".to_string()),
            Text::Attached("s".to_string()),
            Text::TextAction(vec![TextAction::case(false, true)]),
            Text::Lit("hello".to_string()),
            Text::Lit("bite".to_string()),
            Text::Attached("ing".to_string()),
            Text::Attached("s".to_string()),
            Text::TextAction(vec![TextAction::case(true, true)]),
            Text::Attached("ed".to_string()),
        ]);

        assert_eq!(
            translated,
            vec![
                TextInternal::Lit("fancies".to_string()),
                TextInternal::Actions(HashMap::from_iter(vec![(TextActionType::CasePrev, true),])),
                TextInternal::Lit("hello".to_string()),
                TextInternal::Lit("bitings".to_string()),
                TextInternal::Actions(HashMap::from_iter(vec![
                    (TextActionType::CaseNext, true),
                    (TextActionType::SpaceNext, false)
                ])),
                TextInternal::Lit("ed".to_string()),
            ]
        );
    }

    #[test]
    fn test_merge_spaces() {
        let translated = merge_translations(vec![
            Text::Attached(" ".to_string()),
            Text::TextAction(vec![TextAction::space(true, false)]),
            Text::Attached(" ".to_string()),
            Text::TextAction(vec![TextAction::space(true, false)]),
        ]);

        assert_eq!(
            translated,
            vec![
                TextInternal::Actions(HashMap::from_iter(
                    vec![(TextActionType::SpaceNext, false),]
                )),
                TextInternal::Lit(" ".to_string()),
                TextInternal::Actions(HashMap::from_iter(
                    vec![(TextActionType::SpaceNext, false),]
                )),
                TextInternal::Lit(" ".to_string()),
                TextInternal::Actions(HashMap::from_iter(
                    vec![(TextActionType::SpaceNext, false),]
                )),
            ]
        );
    }

    #[test]
    fn test_word_change_first_letter() {
        assert_eq!(word_change_first_letter("hello".to_owned(), true), "Hello");
        assert_eq!(word_change_first_letter("".to_owned(), true), "");
        assert_eq!(word_change_first_letter("Hello".to_owned(), true), "Hello");
    }
}
