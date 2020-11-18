//! Helper functions for finding the difference between 2 translations and turning that into a command.
use std::cmp;

use crate::commands::Command;
use crate::stroke::Stroke;
use crate::translator::TextAction;
use crate::translator::Translation;

mod parser;

/// A translation that does not have a command
#[derive(Debug)]
enum TranslationNoCommand {
    Text(String),
    UnknownStroke(Stroke),
    TextAction(Vec<TextAction>),
}

fn remove_commands(translations: Vec<Translation>) -> Vec<TranslationNoCommand> {
    let mut result = vec![];
    for t in translations {
        match t {
            Translation::Text(text) => result.push(TranslationNoCommand::Text(text)),
            Translation::TextAction(actions) => {
                result.push(TranslationNoCommand::TextAction(actions))
            }
            Translation::UnknownStroke(stroke) => {
                result.push(TranslationNoCommand::UnknownStroke(stroke))
            }
            Translation::Command(_) => {
                // ignore commands
            }
        }
    }

    result
}

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

    // include 1 addition translation in case the first text command needs the previous text
    if i > 0 {
        i -= 1;
    }

    // directly return the new command if that is what differs
    if let Some(Translation::Command(ref cmd)) = new.last() {
        if let Some(old_translation) = old.last() {
            match old_translation {
                // return the new command if it's different than the old
                Translation::Command(ref old_cmd) => {
                    if cmd != old_cmd {
                        return cmd.clone();
                    }
                }
                _ => {
                    return cmd.clone();
                }
            }
        } else {
            // there was no older translation, so directly return the new one
            return cmd.clone();
        }
    }

    // only diff translations starting from where they differ and ignore commands
    let old_no_command: Vec<_> = remove_commands(old[i..].to_owned());
    let new_no_command: Vec<_> = remove_commands(new[i..].to_owned());

    // compare the two and return the result
    text_diff(
        parser::parse_translation(old_no_command),
        parser::parse_translation(new_no_command),
    )
}

/// Compute the command necessary to make the old string into the new
fn text_diff(old: String, new: String) -> Command {
    if old.len() == 0 {
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

    Command::replace_text(old.len() - i, &new[i..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{ExternalCommand, InternalCommand};
    use crate::stroke::Stroke;

    #[test]
    fn test_diff_same() {
        let command = translation_diff(
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("Hi".to_string()),
            ],
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("Hi".to_string()),
            ],
        );

        assert_eq!(command, Command::add_text(""));
    }

    #[test]
    fn test_diff_empty() {
        let command = translation_diff(&vec![], &vec![]);

        assert_eq!(command, Command::add_text(""));
    }

    #[test]
    fn test_diff_one_empty() {
        let command = translation_diff(&vec![], &vec![Translation::Text("Hello".to_string())]);

        assert_eq!(command, Command::add_text(" Hello"));
    }

    #[test]
    fn test_diff_one_command_empty() {
        let command = translation_diff(
            &vec![],
            &vec![Translation::Command(Command::External(
                ExternalCommand::PrintHello,
            ))],
        );

        assert_eq!(command, Command::External(ExternalCommand::PrintHello));
    }

    #[test]
    fn test_diff_simple_add() {
        let command = translation_diff(
            &vec![Translation::Text("Hello".to_string())],
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("Hi".to_string()),
            ],
        );

        assert_eq!(command, Command::add_text(" Hi"));
    }

    #[test]
    fn test_diff_correction() {
        let command = translation_diff(
            &vec![Translation::Text("Hello".to_string())],
            &vec![Translation::Text("He..llo".to_string())],
        );

        assert_eq!(command, Command::replace_text(3, "..llo"));
    }

    #[test]
    fn test_diff_deletion() {
        let command = translation_diff(&vec![Translation::Text("Hello".to_string())], &vec![]);

        assert_eq!(command, Command::replace_text(6, ""));
    }

    #[test]
    fn test_diff_unknown_correction() {
        let command = translation_diff(
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::UnknownStroke(Stroke::new("WUPB")),
            ],
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("Won".to_string()),
            ],
        );

        assert_eq!(command, Command::replace_text(3, "on"));
    }

    #[test]
    fn test_diff_text_actions() {
        let command = translation_diff(
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::TextAction(vec![TextAction::space(true, false)]),
                Translation::Text("world".to_string()),
            ],
            &vec![
                Translation::Text("Hi".to_string()),
                Translation::TextAction(vec![TextAction::case(true, true)]),
                Translation::Text("world".to_string()),
            ],
        );

        assert_eq!(command, Command::replace_text(9, "i World"));
    }

    #[test]
    fn test_diff_prev_word_text_actions() {
        let command = translation_diff(
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("world".to_string()),
            ],
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("world".to_string()),
                Translation::TextAction(vec![TextAction::case(false, true)]),
            ],
        );

        assert_eq!(command, Command::replace_text(5, "World"));
    }

    #[test]
    fn test_diff_same_command() {
        let command = translation_diff(
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
            ],
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
            ],
        );

        assert_eq!(command, Command::add_text(""));
    }

    #[test]
    fn test_diff_repeated_command() {
        let command = translation_diff(
            &vec![
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
            ],
            &vec![
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
            ],
        );

        assert_eq!(command, Command::External(ExternalCommand::PrintHello));
    }

    #[test]
    fn test_diff_external_command() {
        let command = translation_diff(
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("world".to_string()),
            ],
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("world".to_string()),
                Translation::Command(Command::External(ExternalCommand::PrintHello)),
            ],
        );

        assert_eq!(command, Command::External(ExternalCommand::PrintHello));
    }

    #[test]
    fn test_diff_undo() {
        let command = translation_diff(
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("world".to_string()),
            ],
            &vec![
                Translation::Text("Hello".to_string()),
                Translation::Text("world".to_string()),
                Translation::Command(Command::Internal(InternalCommand::Undo)),
            ],
        );

        assert_eq!(command, Command::Internal(InternalCommand::Undo));
    }
}
