use crate::commands::Command;
use crate::stroke::Stroke;
use crate::translator::diff::translation_diff;
use crate::translator::lookup::translate_strokes;
use dictionary::Dictionary;

pub mod dictionary;
mod diff;
mod lookup;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TextAction {
    action_type: TextActionType,
    // associated value for each text action (see TextActionType documentation)
    val: bool,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum TextActionType {
    // true to force a space, false for no space
    SpaceNext,
    SpacePrev,
    // true for uppercase, false for lowercase
    CaseNext,
    CasePrev,
}

impl TextAction {
    pub fn space(is_next: bool, val: bool) -> Self {
        Self {
            action_type: if is_next {
                TextActionType::SpaceNext
            } else {
                TextActionType::SpacePrev
            },
            val,
        }
    }

    pub fn case(is_next: bool, val: bool) -> Self {
        Self {
            action_type: if is_next {
                TextActionType::CaseNext
            } else {
                TextActionType::CasePrev
            },
            val,
        }
    }
}

/// A dictionary entry. It could be a command, in which case it is passed directly to the
/// dispatcher. Otherwise it is something that pertains to text, which is parsed here in translator
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Translation {
    Text(Text),
    Command(Command),
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Text {
    // text literal that can be upper/lower cased
    Lit(String),
    // unknown strokes always printed in all caps
    UnknownStroke(Stroke),
    // an attached string that gets orthographic rules applied
    Attached(String),
    // actions like no space, uppercase; apply to adjacent Texts
    TextAction(Vec<TextAction>),
}

impl Translation {
    fn as_text(&self) -> Option<Text> {
        match self {
            Translation::Text(ref text) => Some(text.clone()),
            _ => None,
        }
    }
}

type DictEntries = Vec<(Stroke, Vec<Translation>)>;
type DictEntry = (Stroke, Vec<Translation>);

#[derive(Debug, PartialEq)]
pub struct State {
    // should only include "undo-able" strokes (this excludes commands)
    prev_strokes: Vec<Stroke>,
}

impl Default for State {
    fn default() -> Self {
        State {
            prev_strokes: vec![],
        }
    }
}

impl State {
    #[cfg(test)]
    pub fn with_strokes(prev_strokes: Vec<Stroke>) -> Self {
        State {
            prev_strokes,
            ..State::default()
        }
    }
}

// most number of strokes to stroke in prev_strokes; limits undo to this many strokes
const MAX_STROKE_BUFFER: usize = 100;

pub fn translate(stroke: Stroke, dict: &Dictionary, mut state: State) -> (Command, State) {
    if state.prev_strokes.len() > MAX_STROKE_BUFFER {
        state.prev_strokes.remove(0);
    }

    let old_translations = translate_strokes(&state.prev_strokes, dict);
    state.prev_strokes.push(stroke);
    let new_translations = translate_strokes(&state.prev_strokes, dict);

    let command = translation_diff(&old_translations, &new_translations);

    (command, state)
}

pub fn undo(dict: &Dictionary, mut state: State) -> (Command, State) {
    let old_translations = translate_strokes(&state.prev_strokes, dict);
    // need to remove two strokes: the one that triggered the undo and the stroke to be undone
    state.prev_strokes.pop();
    state.prev_strokes.pop();
    let new_translations = translate_strokes(&state.prev_strokes, dict);

    let command = translation_diff(&old_translations, &new_translations);

    (command, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_lower_overwrite() {
        // later dictionary entries should overwrite earlier ones
        let dict = Dictionary::new(vec![
            (
                Stroke::new("H-L"),
                vec![Translation::Text(Text::Lit("Hello".to_string()))],
            ),
            (
                Stroke::new("WORLD"),
                vec![Translation::Text(Text::Lit("World".to_string()))],
            ),
            (
                Stroke::new("H-L"),
                vec![Translation::Text(Text::Lit("another".to_string()))],
            ),
        ]);

        let state = State::default();

        let (command, _) = translate(Stroke::new("H-L"), &dict, state);

        assert_eq!(command, Command::add_text(" another"));
    }
}
