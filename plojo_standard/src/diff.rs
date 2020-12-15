//! Helper functions for finding the difference between 2 translations and turning that into a command.
use crate::Translation;
use plojo_core::Command;
use std::cmp;

mod parser;

use parser::parse_translation;

const SPACE: char = ' ';

/// Finds the difference between two translations, converts them to their string representations,
/// and diffs the strings to create a command. Has an option to insert spaces after words instead
/// of before
pub(super) fn translation_diff(
    old: &[Translation],
    new: &[Translation],
    space_after: bool,
) -> Vec<Command> {
    // ignore commands and convert old translations to text
    let old_translations: Vec<_> = old.iter().flat_map(|t| Translation::as_text(t)).collect();
    let old_parsed = parse_translation(old_translations, space_after);

    // if added a command, return that directly
    if old.len() + 1 == new.len() {
        if let Some(Translation::Command {
            cmds,
            suppress_space_before,
            ..
        }) = new.last()
        {
            let mut cmds = cmds.clone();
            // if suppress space, delete the space if there is any
            if *suppress_space_before && old_parsed.chars().last() == Some(SPACE) {
                cmds.insert(0, Command::Replace(1, "".to_string()));
            }
            return cmds;
        }
    }

    // ignore commands and convert old translations to text
    let new_translations: Vec<_> = new.iter().flat_map(|t| Translation::as_text(t)).collect();
    let new_parsed = parse_translation(new_translations, space_after);

    // compare the two and return the result
    vec![text_diff(old_parsed, new_parsed)]
}

/// Compute the command necessary to make the old string into the new
fn text_diff(old: String, new: String) -> Command {
    if old.is_empty() {
        if new.is_empty() {
            return Command::NoOp;
        }

        return Command::add_text(&new);
    }
    if new.is_empty() {
        return Command::replace_text(old.len(), "");
    }

    let old_chars_len = old.clone().chars().count();
    let new_chars_len = new.clone().chars().count();
    let mut old_chars = old.chars();
    let mut new_chars = new.chars();

    // find where the new translations differ from the old
    let mut i: usize = 0;
    let loop_size: usize = cmp::min(old_chars_len, new_chars_len);
    while i < loop_size {
        if old_chars.next() != new_chars.next() {
            break;
        }
        i += 1;
    }

    if i == old_chars_len && old_chars_len == new_chars_len {
        return Command::NoOp;
    }

    Command::replace_text(old_chars_len - i, &new.chars().skip(i).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StateAction, Text, TextAction};
    use plojo_core::Stroke;

    fn translation_diff_space_after(old: &[Translation], new: &[Translation]) -> Vec<Command> {
        translation_diff(old, new, false)
    }

    fn basic_command(cmds: Vec<Command>) -> Translation {
        Translation::Command {
            cmds,
            text_after: None,
            suppress_space_before: false,
        }
    }

    #[test]
    fn test_diff_same() {
        let command = translation_diff_space_after(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("Hi".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("Hi".to_string())),
            ],
        );

        assert_eq!(command, vec![Command::NoOp]);
    }

    #[test]
    fn test_diff_empty() {
        let command = translation_diff_space_after(&vec![], &vec![]);

        assert_eq!(command, vec![Command::NoOp]);
    }

    #[test]
    fn test_diff_one_empty() {
        let command = translation_diff_space_after(
            &vec![],
            &vec![Translation::Text(Text::Lit("Hello".to_string()))],
        );

        assert_eq!(command, vec![Command::add_text(" Hello")]);
    }

    #[test]
    fn test_diff_one_command_empty() {
        let command =
            translation_diff_space_after(&vec![], &vec![basic_command(vec![Command::PrintHello])]);

        assert_eq!(command, vec![Command::PrintHello]);
    }

    #[test]
    fn test_diff_simple_add() {
        let command = translation_diff_space_after(
            &vec![Translation::Text(Text::Lit("Hello".to_string()))],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("Hi".to_string())),
            ],
        );

        assert_eq!(command, vec![Command::add_text(" Hi")]);
    }

    #[test]
    fn test_diff_correction() {
        let command = translation_diff_space_after(
            &vec![Translation::Text(Text::Lit("Hello".to_string()))],
            &vec![Translation::Text(Text::Lit("He..llo".to_string()))],
        );

        assert_eq!(command, vec![Command::replace_text(3, "..llo")]);
    }

    #[test]
    fn test_diff_deletion() {
        let command = translation_diff_space_after(
            &vec![Translation::Text(Text::Lit("Hello".to_string()))],
            &vec![],
        );

        assert_eq!(command, vec![Command::replace_text(6, "")]);
    }

    #[test]
    fn test_diff_unknown_correction() {
        let command = translation_diff_space_after(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::UnknownStroke(Stroke::new("WUPB"))),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("Won".to_string())),
            ],
        );

        assert_eq!(command, vec![Command::replace_text(3, "on")]);
    }

    #[test]
    fn test_diff_text_actions() {
        let command = translation_diff_space_after(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hi".to_string())),
                Translation::Text(Text::StateAction(StateAction::ForceCapitalize)),
                Translation::Text(Text::Lit("world".to_string())),
            ],
        );

        assert_eq!(command, vec![Command::replace_text(10, "i World")]);
    }

    #[test]
    fn test_diff_prev_word_text_actions() {
        let command = translation_diff_space_after(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
                Translation::Text(Text::TextAction(TextAction::CapitalizePrev)),
            ],
        );

        assert_eq!(command, vec![Command::replace_text(5, "World")]);
    }

    #[test]
    fn test_diff_same_command() {
        let command = translation_diff_space_after(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                basic_command(vec![Command::PrintHello]),
                basic_command(vec![Command::PrintHello]),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                basic_command(vec![Command::PrintHello]),
                basic_command(vec![Command::PrintHello]),
            ],
        );

        assert_eq!(command, vec![Command::NoOp]);
    }

    #[test]
    fn test_diff_repeated_command() {
        let command = translation_diff_space_after(
            &vec![
                basic_command(vec![Command::PrintHello]),
                basic_command(vec![Command::PrintHello]),
            ],
            &vec![
                basic_command(vec![Command::PrintHello]),
                basic_command(vec![Command::PrintHello]),
                basic_command(vec![Command::PrintHello]),
            ],
        );

        assert_eq!(command, vec![Command::PrintHello]);
    }

    #[test]
    fn test_diff_external_command() {
        let command = translation_diff_space_after(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
                basic_command(vec![Command::PrintHello]),
            ],
        );

        assert_eq!(command, vec![Command::PrintHello]);
    }

    #[test]
    fn test_unicode() {
        let command = text_diff(
            // note that these are "em dashes"
            " ——a".to_string(),
            " —Ω".to_string(),
        );

        assert_eq!(command, Command::Replace(2, "Ω".to_string()));
    }
}
