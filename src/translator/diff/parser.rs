use crate::translator::diff::TranslationNoCommand;
use crate::translator::TextActionType;
use std::collections::HashMap;

/// For the translation_to_string function
#[derive(Debug, PartialEq)]
enum Text {
    // text literal, such as from a translation or a unknown stroke
    Lit(String),
    // actions that could apply to the previous or the next word and its boolean value (upper/lower, no/space)
    Actions(HashMap<TextActionType, bool>),
}

/// Converts translations into their string representation by adding spaces in between words and
/// applying text actions.
pub(super) fn parse_translation(translations: Vec<TranslationNoCommand>) -> String {
    let mut s = String::from("");
    let mut merged: Vec<_> = merge_translations(translations)
        .into_iter()
        .map(Some)
        .collect();
    // push another None so that all the values have a chance to be `cur`
    merged.push(None);
    merged.push(None);

    // value of the current
    let mut prev: Option<Text> = None;
    let mut cur: Option<Text> = None;
    let mut next: Option<Text> = None;
    for text in merged {
        // if `cur` is a literal string
        if let Some(Text::Lit(ref text_str)) = cur {
            let str = text_str.clone();
            let mut force_space = true;
            let mut uppercase: Option<bool> = None;

            // apply transformations on it by looking ahead and backwards

            // looking backwards first
            if let Some(Text::Actions(actions)) = &prev {
                if let Some(is_force_space) = actions.get(&TextActionType::SpaceNext) {
                    force_space = *is_force_space;
                }
                if let Some(is_uppercase) = actions.get(&TextActionType::CaseNext) {
                    uppercase = Some(*is_uppercase);
                }
            }

            // then look ahead, because things ahead can override things behind
            if let Some(Text::Actions(actions)) = &next {
                if let Some(is_force_space) = actions.get(&TextActionType::SpacePrev) {
                    force_space = *is_force_space;
                }
                if let Some(is_uppercase) = actions.get(&TextActionType::CasePrev) {
                    uppercase = Some(*is_uppercase);
                }
            }

            if force_space {
                s.push(' ');
            }
            if let Some(uppercase) = uppercase {
                s.push_str(&word_change_first_letter(str, uppercase));
            } else {
                s.push_str(&str);
            }
        }

        // advance to next item in vec
        prev = cur;
        cur = next;
        next = text;
    }

    s
}

/// Simplifies a series of translations by merging consequetive text actions into a hashset
fn merge_translations(translations: Vec<TranslationNoCommand>) -> Vec<Text> {
    // tracks the state for the merging the text actions
    struct IterState {
        // accumulated text literals and text actions
        acc: Vec<Text>,
        // consecutive actions are added to this
        actions: Option<HashMap<TextActionType, bool>>,
    }

    // merge all consecutive text actions into a set
    let IterState { mut acc, actions } = translations.into_iter().fold(
        IterState {
            acc: vec![],
            actions: None,
        },
        |mut state, cur| {
            match cur {
                TranslationNoCommand::Text(text) => {
                    if let Some(actions) = state.actions {
                        state.acc.push(Text::Actions(actions));
                        state.actions = None;
                    }

                    state.acc.push(Text::Lit(text));
                }
                TranslationNoCommand::UnknownStroke(stroke) => {
                    if let Some(actions) = state.actions {
                        state.acc.push(Text::Actions(actions));
                        state.actions = None;
                    }

                    state.acc.push(Text::Lit(stroke.to_raw()));
                }
                TranslationNoCommand::TextAction(actions) => match state.actions {
                    Some(mut prev_actions) => {
                        for a in actions {
                            prev_actions.insert(a.action_type, a.val);
                        }
                        state.actions = Some(prev_actions);
                    }
                    None => {
                        let mut new_actions = HashMap::new();
                        for a in actions {
                            new_actions.insert(a.action_type, a.val);
                        }
                        state.actions = Some(new_actions);
                    }
                },
            };
            state
        },
    );
    if let Some(a) = actions {
        acc.push(Text::Actions(a));
    }
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
    use crate::stroke::Stroke;
    use crate::translator::TextAction;
    use std::iter::FromIterator;

    #[test]
    fn test_parse_empty() {
        let translated = parse_translation(vec![]);

        assert_eq!(translated, "");
    }

    #[test]
    fn test_parse_basic() {
        let translated = parse_translation(vec![
            TranslationNoCommand::Text("hello".to_string()),
            TranslationNoCommand::Text("hi".to_string()),
        ]);

        assert_eq!(translated, " hello hi");
    }

    #[test]
    fn test_parse_text_actions() {
        let translated = parse_translation(vec![
            TranslationNoCommand::TextAction(vec![
                TextAction::space(true, false),
                TextAction::case(true, true),
            ]),
            TranslationNoCommand::Text("hello".to_string()),
            TranslationNoCommand::Text("hi".to_string()),
            TranslationNoCommand::TextAction(vec![TextAction::case(true, true)]),
            TranslationNoCommand::Text("FOo".to_string()),
            TranslationNoCommand::Text("bar".to_string()),
            TranslationNoCommand::Text("baZ".to_string()),
            TranslationNoCommand::TextAction(vec![TextAction::case(true, false)]),
            TranslationNoCommand::TextAction(vec![TextAction::space(true, false)]),
            TranslationNoCommand::Text("NICE".to_string()),
            TranslationNoCommand::TextAction(vec![TextAction::space(true, false)]),
            TranslationNoCommand::Text("".to_string()),
            TranslationNoCommand::Text("well done".to_string()),
        ]);

        assert_eq!(translated, "Hello hi FOo bar baZnICE well done");
    }

    #[test]
    fn test_parse_prev_word_text_actions() {
        let translated = parse_translation(vec![
            TranslationNoCommand::Text("hi".to_string()),
            TranslationNoCommand::TextAction(vec![
                TextAction::case(false, true),
                TextAction::case(false, false),
            ]),
            TranslationNoCommand::TextAction(vec![TextAction::case(true, false)]),
            TranslationNoCommand::TextAction(vec![TextAction::case(false, true)]),
            TranslationNoCommand::Text("FOo".to_string()),
            TranslationNoCommand::Text("bar".to_string()),
            TranslationNoCommand::TextAction(vec![
                TextAction::space(false, false),
                TextAction::case(false, true),
            ]),
            TranslationNoCommand::Text("hello".to_string()),
            TranslationNoCommand::Text("Hi a".to_string()),
            TranslationNoCommand::TextAction(vec![
                TextAction::space(true, true),
                TextAction::case(true, false),
            ]),
            TranslationNoCommand::TextAction(vec![TextAction::case(false, true)]),
            TranslationNoCommand::TextAction(vec![
                TextAction::case(true, true),
                TextAction::case(true, true),
            ]),
            TranslationNoCommand::Text("nice".to_string()),
            TranslationNoCommand::UnknownStroke(Stroke::new("TP-TDZ")),
            TranslationNoCommand::TextAction(vec![
                TextAction::case(false, false),
                TextAction::space(false, false),
            ]),
            TranslationNoCommand::Text("nice".to_string()),
            TranslationNoCommand::TextAction(vec![TextAction::space(true, false)]),
            TranslationNoCommand::Text("another".to_string()),
            TranslationNoCommand::TextAction(vec![TextAction::space(false, true)]),
        ]);

        assert_eq!(translated, " Hi fOoBar hello Hi a NicetP-TDZ nice another");
    }

    #[test]
    fn test_parse_line_start() {
        let translated = parse_translation(vec![
            TranslationNoCommand::TextAction(vec![
                TextAction::space(true, false),
                TextAction::case(true, true),
            ]),
            TranslationNoCommand::Text("hello".to_string()),
            TranslationNoCommand::Text("hi".to_string()),
        ]);

        assert_eq!(translated, "Hello hi");
    }

    #[test]
    fn test_merge_translations() {
        let translated = merge_translations(vec![
            TranslationNoCommand::TextAction(vec![
                TextAction::space(true, true),
                TextAction::case(true, true),
            ]),
            TranslationNoCommand::Text("hello".to_string()),
            TranslationNoCommand::Text("hi".to_string()),
            TranslationNoCommand::TextAction(vec![TextAction::case(true, false)]),
            TranslationNoCommand::TextAction(vec![
                TextAction::case(false, true),
                TextAction::case(false, true),
                TextAction::case(true, false),
            ]),
            TranslationNoCommand::TextAction(vec![
                TextAction::case(false, true),
                TextAction::space(false, false),
                TextAction::case(true, false),
            ]),
            TranslationNoCommand::Text("FOo".to_string()),
            TranslationNoCommand::TextAction(vec![
                TextAction::case(false, true),
                TextAction::case(false, false),
            ]),
            TranslationNoCommand::TextAction(vec![TextAction::case(false, false)]),
            TranslationNoCommand::Text("FOo".to_string()),
            TranslationNoCommand::TextAction(vec![
                TextAction::case(true, false),
                TextAction::case(true, true),
            ]),
        ]);

        assert_eq!(
            translated,
            vec![
                Text::Actions(HashMap::from_iter(vec![
                    (TextActionType::SpaceNext, true),
                    (TextActionType::CaseNext, true),
                ])),
                Text::Lit("hello".to_string()),
                Text::Lit("hi".to_string()),
                Text::Actions(HashMap::from_iter(vec![
                    (TextActionType::CasePrev, true),
                    (TextActionType::SpacePrev, false),
                    (TextActionType::CaseNext, false),
                ])),
                Text::Lit("FOo".to_string()),
                Text::Actions(HashMap::from_iter(vec![(TextActionType::CasePrev, false),])),
                Text::Lit("FOo".to_string()),
                Text::Actions(HashMap::from_iter(vec![(TextActionType::CaseNext, true),])),
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
