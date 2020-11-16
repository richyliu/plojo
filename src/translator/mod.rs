use std::collections::HashMap;

use crate::commands::Command;
use crate::stroke::Stroke;
use crate::translator::diff::translation_diff;
use crate::translator::lookup::translate_strokes;

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
    Command(Vec<Command>),
}

pub struct Dictionary {
    strokes: HashMap<Stroke, Translation>,
}

type DictEntries = Vec<(Stroke, Translation)>;

impl Dictionary {
    pub fn new(entries: DictEntries) -> Self {
        let mut hashmap = HashMap::new();
        for (stroke, command) in entries.into_iter() {
            hashmap.insert(stroke, command);
        }

        Dictionary { strokes: hashmap }
    }

    fn get(&self, stroke: &Stroke) -> Option<Translation> {
        self.strokes.get(stroke).cloned()
    }

    fn lookup(&self, strokes: &[Stroke]) -> Option<Translation> {
        // combine strokes with a `/` between them
        let mut combined = strokes
            .into_iter()
            .map(|s| s.clone().to_raw())
            .fold(String::new(), |acc, s| acc + &s + "/");
        // remove trailing `/`
        combined.pop();

        self.get(&Stroke::new(&combined)).clone()
    }
}

#[derive(Debug, PartialEq)]
pub struct State {
    // should only include "undo-able" strokes (this excludes commands)
    prev_strokes: Vec<Stroke>,
    // text actions to be inserted before the initial strokes
    initial_text_actions: Vec<TextAction>,
}

impl Default for State {
    fn default() -> Self {
        State {
            prev_strokes: vec![],
            initial_text_actions: vec![TextAction::space(true, true), TextAction::case(true, true)],
        }
    }
}

impl State {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_lower_overwrite() {
        let dict = Dictionary::new(vec![
            (Stroke::new("H-L"), Translation::Text("Hello".to_string())),
            (Stroke::new("WORLD"), Translation::Text("World".to_string())),
            (Stroke::new("H-L"), Translation::Text("another".to_string())),
        ]);

        let state = State::default();

        let (command, _) = translate(Stroke::new("H-L"), &dict, state);

        assert_eq!(command, Command::add_text(" another"));
    }
}
