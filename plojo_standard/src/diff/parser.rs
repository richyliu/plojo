use crate::{StateAction, Text, TextAction};
use orthography::apply_orthography;
use regex::Regex;
use std::char;

mod orthography;

lazy_static! {
    // whether a translation contains only digits or the center dash
    // although the regex will mark "-" as a number, such a stroke is not possible
    static ref NUMBER_TRANSLATION_REGEX: Regex = Regex::new(r"^[0-9\-]+$").unwrap();
    // whether a translation contains only digits, in which case it will be glued
    static ref NUMBERS_ONLY_REGEX: Regex = Regex::new(r"^[0-9]+$").unwrap();
}

#[derive(Debug, Default)]
struct State {
    suppress_space: bool,
    force_capitalize: bool,
    prev_is_glued: bool,
}

/// Converts translations into their string representation by adding spaces in between words and
/// applying text actions.
///
/// A state of the spaces/capitalization is kept as it loops over the Texts to build the string.
/// StateActions change that state
pub(super) fn parse_translation(translations: Vec<Text>) -> String {
    // current state
    let mut state: State = Default::default();
    let mut str = String::new();

    for t in translations {
        let next_word;
        let mut next_state: State = Default::default();

        match t {
            Text::Lit(text) => {
                next_word = text.clone();
                // glue it if it is a number stroke
                if NUMBERS_ONLY_REGEX.is_match(&next_word) {
                    next_state.prev_is_glued = true;
                    if state.prev_is_glued {
                        state.suppress_space = true;
                    }
                }
            }
            Text::UnknownStroke(stroke) => {
                let raw_stroke = stroke.to_raw();
                // glue it if it is a number stroke
                if NUMBER_TRANSLATION_REGEX.is_match(&raw_stroke) {
                    // remove the hyphen
                    next_word = raw_stroke.replace("-", "");
                    next_state.prev_is_glued = true;
                    if state.prev_is_glued {
                        state.suppress_space = true;
                    }
                } else {
                    next_word = raw_stroke;
                }
            }
            Text::Attached {
                text,
                joined_next,
                do_orthography,
                carry_capitalization,
            } => {
                next_word = text.clone();
                if joined_next {
                    next_state.suppress_space = true;
                }
                if carry_capitalization {
                    // carry on the capitalization state to the next word
                    next_state.force_capitalize = state.force_capitalize;
                    // don't capitalize this word
                    state.force_capitalize = false;
                }
                // Some means to join stroke to previous word
                if let Some(do_ortho) = do_orthography {
                    state.suppress_space = true;

                    // do orthography rule
                    if do_ortho {
                        let index = find_last_word(&str);
                        // find the last word and apply orthography rule with the suffix
                        if index < str.len() {
                            let new_word = apply_orthography(&str[index..], &text);
                            // replace that word with the new (orthography'ed) one
                            str = str[..index].to_string() + &new_word;
                        } else {
                            // there was no last word, directly add the text
                            str = str + &text;
                        }
                        state = next_state;
                        continue;
                    }
                }
            }
            Text::Glued(text) => {
                next_word = text.clone();
                next_state.prev_is_glued = true;
                if state.prev_is_glued {
                    state.suppress_space = true;
                }
            }
            Text::StateAction(action) => {
                match action {
                    StateAction::ForceCapitalize => {
                        state.force_capitalize = true;
                    }
                    StateAction::SuppressSpace => {
                        state.suppress_space = true;
                    }
                }
                continue;
            }
            Text::TextAction(action) => {
                str = perform_text_action(&str, action);
                continue;
            }
        }

        if !state.suppress_space {
            str.push(' ');
        }
        if state.force_capitalize {
            str.push_str(&word_change_first_letter(next_word));
        } else {
            str.push_str(&next_word);
        }

        state = next_state;
    }

    str
}

/// Forces the first letter of a string to be uppercase
fn word_change_first_letter(text: String) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Find the index in the text after the last space
/// This index is 0 if there is no whitespace, and text.len() if the last char is a whitespace
fn find_last_word(text: &str) -> usize {
    if let Some(i) = text.rfind(char::is_whitespace) {
        // add 1 to remove the space
        // whitespace takes up 1 byte, so adding 1 is safe here
        i + 1
    } else {
        // no whitespace, so everything must be a word
        0
    }
}

// chars (besides alphanumeric) that are considered part of a word
// This is used for deciding what is a word when capitalizing the previous word
const WORD_CHARS: [char; 2] = ['-', '_'];

fn perform_text_action(text: &str, action: TextAction) -> String {
    match action {
        TextAction::SuppressSpacePrev => {
            let mut new_str = text.to_string();
            let index = find_last_word(&text);
            // find the last word and see if there is a space before it
            if index > 0 && text.get(index - 1..index) == Some(" ") {
                // remove the space (this is safe because we checked the index above)
                new_str.remove(index - 1);
            }
            new_str
        }
        TextAction::CapitalizePrev => {
            // find the last non-alphanumeric (nor hyphen) character
            let index = if let Some(i) =
                text.rfind(|c| !(char::is_alphanumeric(c) || WORD_CHARS.contains(&c)))
            {
                // size of whatever char was before the word
                // unwrap is safe because we found the index `i` with rfind
                let char_size = text[i..].chars().next().unwrap().to_string().len();
                // add to get to the next char (the actual word)
                i + char_size
            } else {
                // no whitespace, so everything must be a word
                0
            };

            // capitalize the last word
            let word = text[index..].to_string();
            let capitalized = word_change_first_letter(word);
            text[..index].to_string() + &capitalized
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StateAction, TextAction};
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
            Text::StateAction(StateAction::SuppressSpace),
            Text::StateAction(StateAction::ForceCapitalize),
            Text::Lit("hello".to_string()),
            Text::Lit("hi".to_string()),
            Text::StateAction(StateAction::ForceCapitalize),
            Text::Lit("FOo".to_string()),
            Text::Lit("bar".to_string()),
            Text::Lit("baZ".to_string()),
            Text::StateAction(StateAction::SuppressSpace),
            Text::Lit("NICE".to_string()),
            Text::StateAction(StateAction::SuppressSpace),
            Text::Lit("".to_string()),
            Text::Lit("well done".to_string()),
        ]);

        assert_eq!(translated, "Hello hi FOo bar baZNICE well done");
    }

    #[test]
    fn test_parse_prev_word_text_actions() {
        let translated = parse_translation(vec![
            Text::Lit("hi".to_string()),
            Text::TextAction(TextAction::CapitalizePrev),
            Text::TextAction(TextAction::CapitalizePrev),
            Text::Lit("FOo".to_string()),
            Text::Lit("bar".to_string()),
            Text::TextAction(TextAction::SuppressSpacePrev),
            Text::TextAction(TextAction::CapitalizePrev),
            Text::Lit("hello".to_string()),
            Text::Lit("Hi a".to_string()),
            Text::TextAction(TextAction::CapitalizePrev),
            Text::StateAction(StateAction::ForceCapitalize),
            Text::Lit("nice".to_string()),
            Text::UnknownStroke(Stroke::new("TP-TDZ")),
            Text::TextAction(TextAction::SuppressSpacePrev),
            Text::Lit("nice".to_string()),
            Text::StateAction(StateAction::SuppressSpace),
            Text::Lit("another".to_string()),
        ]);

        assert_eq!(translated, " Hi FOobar hello Hi A NiceTP-TDZ niceanother");
    }

    #[test]
    fn test_parse_line_start() {
        let translated = parse_translation(vec![
            Text::StateAction(StateAction::SuppressSpace),
            Text::StateAction(StateAction::ForceCapitalize),
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
        ]);

        assert_eq!(translated, " hello hihi foo twothree");
    }

    #[test]
    fn test_word_change_first_letter() {
        assert_eq!(word_change_first_letter("hello".to_owned()), "Hello");
        assert_eq!(word_change_first_letter("".to_owned()), "");
        assert_eq!(word_change_first_letter("Hello".to_owned()), "Hello");
    }

    #[test]
    fn test_unicode() {
        let translated = parse_translation(vec![
            Text::Lit("hi".to_string()),
            Text::Lit("hello".to_string()),
            Text::Lit("𐀀".to_string()),
            Text::TextAction(TextAction::SuppressSpacePrev),
            Text::Lit("©aa".to_string()),
            Text::TextAction(TextAction::CapitalizePrev),
            Text::TextAction(TextAction::SuppressSpacePrev),
        ]);

        assert_eq!(translated, " hi hello𐀀©Aa");
    }

    #[test]
    fn test_double_space() {
        let translated = parse_translation(vec![
            Text::Lit("hello".to_string()),
            Text::Attached {
                text: " ".to_string(),
                joined_next: true,
                do_orthography: Some(true),
                carry_capitalization: false,
            },
            Text::Attached {
                text: " ".to_string(),
                joined_next: true,
                do_orthography: Some(true),
                carry_capitalization: false,
            },
        ]);

        assert_eq!(translated, " hello  ");
    }

    #[test]
    fn test_find_last_word() {
        assert_eq!(find_last_word("hello world"), 6);
        assert_eq!(find_last_word(" world"), 1);
        assert_eq!(find_last_word("test "), 5);
        assert_eq!(find_last_word("nospace"), 0);
        assert_eq!(find_last_word(" there are many words"), 16);
    }

    #[test]
    fn test_perform_text_action() {
        assert_eq!(
            perform_text_action("foo bar", TextAction::SuppressSpacePrev),
            "foobar"
        );
        assert_eq!(
            perform_text_action(" hello", TextAction::CapitalizePrev),
            " Hello"
        );
        assert_eq!(
            perform_text_action(" there are many words", TextAction::CapitalizePrev),
            " there are many Words"
        );
        assert_eq!(
            perform_text_action(" no previous word ", TextAction::CapitalizePrev),
            " no previous word "
        );
        assert_eq!(
            perform_text_action(" ∅∅byteboundary", TextAction::CapitalizePrev),
            " ∅∅Byteboundary"
        );
        assert_eq!(
            // This weird character becomes 2 S's when capitalized
            perform_text_action(" ßweird_char", TextAction::CapitalizePrev),
            " SSweird_char"
        );
        assert_eq!(
            perform_text_action(" (symbol", TextAction::CapitalizePrev),
            " (Symbol"
        );
        assert_eq!(
            perform_text_action(" !symbol-hyphen", TextAction::CapitalizePrev),
            " !Symbol-hyphen"
        );
    }

    #[test]
    fn test_carry_capitalization() {
        let translated = parse_translation(vec![
            Text::Lit("fairy".to_string()),
            Text::StateAction(StateAction::ForceCapitalize),
            Text::Attached {
                text: "s".to_string(),
                joined_next: false,
                do_orthography: Some(true),
                carry_capitalization: true,
            },
            Text::Attached {
                text: "b".to_string(),
                joined_next: true,
                do_orthography: None,
                carry_capitalization: true,
            },
            Text::Lit("hi".to_string()),
        ]);

        assert_eq!(translated, " fairies bHi");
    }
}
