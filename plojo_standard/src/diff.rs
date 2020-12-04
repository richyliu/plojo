//! Helper functions for finding the difference between 2 translations and turning that into a command.
use crate::Translation;
use plojo_core::Command;
use std::cmp;

mod parser;

/// Finds the difference between two translations, converts them to their string representations,
/// and diffs the strings to create a command
pub(super) fn translation_diff(old: &Vec<Translation>, new: &Vec<Translation>) -> Vec<Command> {
    // if added a command, return that directly
    if old.len() + 1 == new.len() {
        if let Some(Translation::Command {
            cmds: ref cmd,
            text_actions: _,
        }) = new.last()
        {
            // TODO: handle text_actions after the command
            return cmd.clone();
        }
    }

    // only diff translations starting from where they differ and ignore commands
    let old: Vec<_> = old.iter().filter_map(Translation::as_text).collect();
    let new: Vec<_> = new.iter().filter_map(Translation::as_text).collect();

    // find where the new translations differ from the old
    let mut i = 0;
    let loop_size = cmp::min(old.len(), new.len());
    while i < loop_size {
        if old.get(i) != new.get(i) {
            break;
        }
        i += 1;
    }

    // include 2 additional translations
    // in case the first text command needs the previous text or two commands (one in front and one
    // behind a word) target the same word
    if i > 1 {
        i -= 2;
    } else {
        i = 0;
    }

    // only consider the translations after where they differ
    let old = old[i..].to_vec();
    let new = new[i..].to_vec();

    // compare the two and return the result
    vec![text_diff(
        parser::parse_translation(old),
        parser::parse_translation(new),
    )]
}

/// Compute the command necessary to make the old string into the new
fn text_diff(old: String, new: String) -> Command {
    if old.len() == 0 {
        if new.len() == 0 {
            return Command::NoOp;
        }

        return Command::add_text(&new);
    }
    if new.len() == 0 {
        return Command::replace_text(old.len(), "");
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

    if i == old.len() && old.len() == new.len() {
        return Command::NoOp;
    }

    Command::replace_text(old.len() - i, &new[i..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Text, TextAction};
    use plojo_core::Stroke;

    #[test]
    fn test_diff_same() {
        let command = translation_diff(
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
        let command = translation_diff(&vec![], &vec![]);

        assert_eq!(command, vec![Command::NoOp]);
    }

    #[test]
    fn test_diff_one_empty() {
        let command = translation_diff(
            &vec![],
            &vec![Translation::Text(Text::Lit("Hello".to_string()))],
        );

        assert_eq!(command, vec![Command::add_text(" Hello")]);
    }

    #[test]
    fn test_diff_one_command_empty() {
        let command = translation_diff(
            &vec![],
            &vec![Translation::Command {
                cmds: vec![Command::PrintHello],
                text_actions: None,
            }],
        );

        assert_eq!(command, vec![Command::PrintHello]);
    }

    #[test]
    fn test_diff_simple_add() {
        let command = translation_diff(
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
        let command = translation_diff(
            &vec![Translation::Text(Text::Lit("Hello".to_string()))],
            &vec![Translation::Text(Text::Lit("He..llo".to_string()))],
        );

        assert_eq!(command, vec![Command::replace_text(3, "..llo")]);
    }

    #[test]
    fn test_diff_deletion() {
        let command = translation_diff(
            &vec![Translation::Text(Text::Lit("Hello".to_string()))],
            &vec![],
        );

        assert_eq!(command, vec![Command::replace_text(6, "")]);
    }

    #[test]
    fn test_diff_unknown_correction() {
        let command = translation_diff(
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
        let command = translation_diff(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit("world".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hi".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::case(true, true)])),
                Translation::Text(Text::Lit("world".to_string())),
            ],
        );

        assert_eq!(command, vec![Command::replace_text(9, "i World")]);
    }

    #[test]
    fn test_diff_prev_word_text_actions() {
        let command = translation_diff(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::case(false, true)])),
            ],
        );

        assert_eq!(command, vec![Command::replace_text(5, "World")]);
    }

    #[test]
    fn test_diff_same_command() {
        let command = translation_diff(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
            ],
        );

        assert_eq!(command, vec![Command::NoOp]);
    }

    #[test]
    fn test_diff_repeated_command() {
        let command = translation_diff(
            &vec![
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
            ],
            &vec![
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
            ],
        );

        assert_eq!(command, vec![Command::PrintHello]);
    }

    #[test]
    fn test_diff_external_command() {
        let command = translation_diff(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::Lit("world".to_string())),
                Translation::Command {
                    cmds: vec![Command::PrintHello],
                    text_actions: None,
                },
            ],
        );

        assert_eq!(command, vec![Command::PrintHello]);
    }

    #[test]
    fn test_text_actions_no_double() {
        let command = translation_diff(
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit(",".to_string())),
            ],
            &vec![
                Translation::Text(Text::Lit("Hello".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(true, false)])),
                Translation::Text(Text::Lit(",".to_string())),
                Translation::Text(Text::TextAction(vec![TextAction::space(false, false)])),
            ],
        );

        assert_eq!(command, vec![Command::NoOp]);
    }
}
