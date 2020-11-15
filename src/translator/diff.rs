//! Helper functions for finding the difference between 2 translations and turning that into a command.
use std::cmp;

use crate::commands::Command;
use crate::translator::TextAction;
use crate::translator::Translation;

/// Finds the difference between two translations, converts them to their string representations,
/// and diffs the strings to create a command
pub fn translation_diff(old: &Vec<Translation>, new: &Vec<Translation>) -> Command {
    // find where the new translations differ from the old
    let mut i = 0;
    let loop_size = cmp::min(old.len(), new.len());
    while i < loop_size {
        if old.get(i) != new.get(i) {
            break;
        }
        i += 1;
    }

    // starting from where the translations differ, ignore any none text command
    let old_no_command: Vec<_> = old[i..].iter().filter(|t| !t.is_command()).collect();
    let new_no_command: Vec<_> = new[i..].iter().filter(|t| !t.is_command()).collect();
    // TODO: return the command directly if that is what is different

    // compare the two and return the result
    text_diff(
        translation_to_string(old_no_command),
        translation_to_string(new_no_command),
    )
}

/// Converts translations into their string representation by adding spaces in between words and
/// applying text actions.
///
/// Can only add space before each word
fn translation_to_string(translations: Vec<&Translation>) -> String {
    let mut s = String::from("");

    let mut next_add_space = true;
    // force first letter of next word to be upper (true) or lower (false)
    // None represents no forcing
    let mut next_force_upper: Option<bool> = None;
    for t in translations {
        match t {
            Translation::Text(text) => {
                if next_add_space {
                    s.push_str(" ");
                }

                if let Some(upper) = next_force_upper {
                    s.push_str(&word_change_first_letter(text.to_owned(), upper));
                } else {
                    s.push_str(text);
                }

                next_add_space = true;
                next_force_upper = None;
            }
            Translation::UnknownStroke(stroke) => {
                if next_add_space {
                    s.push_str(" ");
                }
                s.push_str(&stroke.clone().to_raw());
                next_add_space = true;
                next_force_upper = None;
            }
            Translation::TextAction(actions) => {
                for action in actions {
                    match action {
                        TextAction::NoSpace => {
                            next_add_space = false;
                        }
                        TextAction::ForceSpace => {
                            next_add_space = true;
                        }
                        TextAction::LowercasePrev => {
                            panic!("actions on previous words not yet supported");
                        }
                        TextAction::UppercasePrev => {
                            panic!("actions on previous words not yet supported");
                        }
                        TextAction::LowercaseNext => {
                            next_force_upper = Some(false);
                        }
                        TextAction::UppercaseNext => {
                            next_force_upper = Some(true);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    s
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

/// Diffs two strings, creating a command to make the old into the new
fn text_diff(old: String, new: String) -> Command {
    if old.len() == 0 {
        return Command::add_text(&new);
    }
    if new.len() == 0 {
        return Command::add_text(&old);
    }

    let mut old_chars = old.chars();
    let mut new_chars = new.chars();

    // find where the new translations differ from the old
    let mut i: usize = 0;
    let loop_size: usize = cmp::min(old.len(), new.len());
    while i < loop_size {
        if old_chars.next() != new_chars.next() {
            break;
        }
        i += 1;
    }

    Command::replace_text(old.len() - i, &new[i..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_diff_same() {
        let command = translation_diff(
            &vec![Translation::text("Hello"), Translation::text("Hi")],
            &vec![Translation::text("Hello"), Translation::text("Hi")],
        );

        assert_eq!(command, Command::add_text(""));
    }

    #[test]
    fn test_translation_diff_empty() {
        let command = translation_diff(&vec![], &vec![]);

        assert_eq!(command, Command::add_text(""));
    }

    #[test]
    fn test_translation_diff_simple_add() {
        let command = translation_diff(
            &vec![Translation::text("Hello")],
            &vec![Translation::text("Hello"), Translation::text("Hi")],
        );

        assert_eq!(command, Command::add_text(" Hi"));
    }

    #[test]
    fn test_translation_to_string_basic() {
        let translated =
            translation_to_string(vec![&Translation::text("hello"), &Translation::text("hi")]);

        assert_eq!(translated, " hello hi");
    }

    #[test]
    fn test_translation_to_string_text_commands() {
        let translated = translation_to_string(vec![
            &Translation::TextAction(vec![TextAction::NoSpace, TextAction::UppercaseNext]),
            &Translation::text("hello"),
            &Translation::text("hi"),
            &Translation::TextAction(vec![TextAction::UppercaseNext]),
            &Translation::text("FOo"),
            &Translation::text("bar"),
            &Translation::text("baZ"),
            &Translation::TextAction(vec![TextAction::LowercaseNext]),
            &Translation::TextAction(vec![TextAction::NoSpace]),
            &Translation::text("NICE"),
            &Translation::TextAction(vec![TextAction::NoSpace]),
            &Translation::text(""),
            &Translation::text("well done"),
        ]);

        assert_eq!(translated, "Hello hi FOo bar baZnICE well done");
    }

    #[test]
    fn test_translation_to_string_line_start() {
        let translated = translation_to_string(vec![
            &Translation::TextAction(vec![TextAction::NoSpace, TextAction::UppercaseNext]),
            &Translation::text("hello"),
            &Translation::text("hi"),
        ]);

        assert_eq!(translated, "Hello hi");
    }

    #[test]
    fn test_word_change_first_letter() {
        assert_eq!(word_change_first_letter("hello".to_owned(), true), "Hello");
        assert_eq!(word_change_first_letter("".to_owned(), true), "");
        assert_eq!(word_change_first_letter("Hello".to_owned(), true), "Hello");
    }
}
