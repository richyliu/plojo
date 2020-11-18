use std::collections::HashMap;

use crate::commands::Command;
use crate::stroke::Stroke;
use crate::translator::diff::translation_diff;
use crate::translator::lookup::translate_strokes;
use std::iter::FromIterator;

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
/// dispatcher. Otherwise it is a text/text action, which is parsed here in the translator
#[derive(Debug, PartialEq, Clone)]
pub enum Translation {
    Text(String),
    UnknownStroke(Stroke),
    TextAction(Vec<TextAction>),
    Command(Command),
}

pub struct Dictionary {
    strokes: HashMap<Stroke, Vec<Translation>>,
}

type DictEntries = Vec<(Stroke, Vec<Translation>)>;
type DictEntry = (Stroke, Vec<Translation>);

impl Dictionary {
    pub fn new(entries: DictEntries) -> Self {
        Self::from_iter(entries.into_iter())
    }

    fn lookup(&self, strokes: &[Stroke]) -> Option<Vec<Translation>> {
        // combine strokes with a `/` between them
        let mut combined = strokes
            .into_iter()
            .map(|s| s.clone().to_raw())
            .fold(String::new(), |acc, s| acc + &s + "/");
        // remove trailing `/`
        combined.pop();

        self.strokes.get(&Stroke::new(&combined)).cloned()
    }
}

impl FromIterator<DictEntry> for Dictionary {
    fn from_iter<T: IntoIterator<Item = DictEntry>>(iter: T) -> Self {
        let mut hashmap: HashMap<Stroke, Vec<Translation>> = HashMap::new();
        for (stroke, command) in iter {
            hashmap.insert(stroke, command);
        }

        Dictionary { strokes: hashmap }
    }
}

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

pub fn translate(stroke: Stroke, dict: &Dictionary, mut state: State) -> (Command, State) {
    let old_translations = translate_strokes(&state.prev_strokes, dict);
    state.prev_strokes.push(stroke);
    let new_translations = translate_strokes(&state.prev_strokes, dict);

    let command = translation_diff(&old_translations, &new_translations);

    (command, state)
}

pub fn undo(dict: &Dictionary, mut state: State) -> (Command, State) {
    let old_translations = translate_strokes(&state.prev_strokes, dict);
    // TODO: undo should remove all strokes until the next non-command stroke
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
                vec![Translation::Text("Hello".to_string())],
            ),
            (
                Stroke::new("WORLD"),
                vec![Translation::Text("World".to_string())],
            ),
            (
                Stroke::new("H-L"),
                vec![Translation::Text("another".to_string())],
            ),
        ]);

        let state = State::default();

        let (command, _) = translate(Stroke::new("H-L"), &dict, state);

        assert_eq!(command, Command::add_text(" another"));
    }
}
