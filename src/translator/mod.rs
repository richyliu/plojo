use std::cmp;
use std::collections::HashMap;

use crate::commands as cmds;

/// A stroke can be a single stroke (ex: "H-L") or several strokes (ex:
/// "H-L/WORLD")
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Stroke(String);

impl Stroke {
    fn new(stroke: &str) -> Self {
        Self(String::from(stroke))
    }

    fn empty_stroke() -> Self {
        Self::new("")
    }

    fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    fn to_raw(self) -> String {
        self.0
    }

    /// Join a copy of two strokes together with a `/` in the middle. If either stroke is empty,
    /// return the other stroke
    fn join(&self, other: &Self) -> Self {
        if other.0.len() == 0 {
            (*self).clone()
        } else if self.0.len() == 0 {
            (*other).clone()
        } else {
            Self::new(&format!("{}/{}", self.0, other.0))
        }
    }
}

/// What action should be taken
#[derive(Debug, PartialEq)]
pub enum Output {
    Replace(usize, String),
    Command(cmds::Command),
}

impl Output {
    pub fn add_text(output: &str) -> Self {
        Self::replace(0, output)
    }
    pub fn replace(backspace_num: usize, replace_str: &str) -> Self {
        Self::Replace(backspace_num, replace_str.to_owned())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TextAction {
    NoSpace,
    ForceSpace,
    LowercasePrev,
    LowercaseNext,
    UppercasePrev,
    UppercaseNext,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Translation {
    Text(String),
    UnknownStroke(Stroke),
    TextAction(Vec<TextAction>),
    Command(Vec<cmds::Command>),
}

impl Translation {
    pub fn text(t: &str) -> Self {
        Self::Text(t.to_owned())
    }

    pub fn is_command(&self) -> bool {
        match self {
            Translation::Command(_) => true,
            _ => false,
        }
    }
}

pub struct Dictionary {
    strokes: HashMap<Stroke, Translation>,
}

type DictEntries = Vec<(Stroke, Translation)>;

impl Dictionary {
    pub fn new(entries: DictEntries) -> Self {
        let mut hashmap = HashMap::new();
        for (stroke, output) in entries.into_iter() {
            hashmap.insert(stroke, output);
        }

        Dictionary { strokes: hashmap }
    }

    fn get(&self, stroke: &Stroke) -> Option<Translation> {
        if let Some(translation) = self.strokes.get(stroke) {
            Some((*translation).clone())
        } else {
            None
        }
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
            initial_text_actions: vec![TextAction::NoSpace, TextAction::UppercaseNext],
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

pub fn translate(stroke: Stroke, dict: &Dictionary, mut state: State) -> (Output, State) {
    let old_translations = translate_strokes(&state.prev_strokes, dict);
    state.prev_strokes.push(stroke);
    let new_translations = translate_strokes(&state.prev_strokes, dict);

    let output = translation_diff(&old_translations, &new_translations);

    (output, state)
}

/// Looks up the definition of strokes in the dictionary, converting them into a Translation. Since
/// multiple strokes map to one dictionary translation, a greedy algorithm is used starting from
/// the oldest strokes
fn translate_strokes(strokes: &Vec<Stroke>, dict: &Dictionary) -> Vec<Translation> {
    println!("strokes: {:?}", strokes);
    // TODO: instead of combining strokes here, do it in the `dict.get`
    let mut strokes_combined = Stroke::empty_stroke();
    let mut translations: Vec<Translation> = vec![];

    let mut longest_translation: Option<Translation> = None;
    for s in strokes {
        let strokes_combined_with_new = strokes_combined.join(s);
        if let Some(translation) = dict.get(&strokes_combined_with_new) {
            longest_translation = Some(translation);
            strokes_combined = strokes_combined_with_new;
        } else {
            if strokes_combined.is_empty() {
                translations.push(Translation::UnknownStroke((*s).clone()));
            } else {
                if let Some(translation) = longest_translation {
                    translations.push(translation);
                    strokes_combined = (*s).clone();
                    longest_translation = dict.get(s);
                } else {
                    panic!("Empty translation! This state is not possible");
                }
            }
        }
    }

    if let Some(translation) = longest_translation {
        translations.push(translation);
    }

    translations
}

/// Finds the difference between two translations, converts them to their string representations,
/// and diffs the strings to create an output
fn translation_diff(old: &Vec<Translation>, new: &Vec<Translation>) -> Output {
    // find where the new translations differ from the old
    let mut i = 0;
    let loop_size = cmp::min(old.len(), new.len());
    while i < loop_size {
        if old.get(i) != new.get(i) {
            break;
        }
        i += 1;
    }

    // starting from where the translations differ, ignore any commands
    let old_no_command: Vec<_> = old[i..].iter().filter(|t| !t.is_command()).collect();
    let new_no_command: Vec<_> = new[i..].iter().filter(|t| !t.is_command()).collect();

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
        println!("t: {:?}, add space: {:?}", t, next_add_space);
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
    if word.len() == 0 {
        return word;
    }

    if let Some(first_letter) = word.get(0..1) {
        let result = if uppercase {
            first_letter.to_uppercase()
        } else {
            first_letter.to_lowercase()
        };

        let mut s = result.to_string();
        if let Some(rest) = word.get(1..) {
            s.push_str(rest);
            return s;
        }
    }
    panic!("oop");
}

/// Diffs two strings, creating a output to make the old into the new
fn text_diff(old: String, new: String) -> Output {
    if old.len() == 0 {
        return Output::Replace(0, new);
    }
    if new.len() == 0 {
        return Output::Replace(0, old);
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

    Output::replace(old.len() - i, &new[i..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_dict() -> Dictionary {
        Dictionary::new(vec![
            (Stroke::new("H-L"), Translation::text("Hello")),
            (Stroke::new("WORLD"), Translation::text("World")),
            (Stroke::new("H-L/A"), Translation::text("He..llo")),
            (Stroke::new("A"), Translation::text("Wrong thing")),
        ])
    }

    #[test]
    fn test_dict_lower_overwrite() {
        let dict = Dictionary::new(vec![
            (Stroke::new("H-L"), Translation::text("Hello")),
            (Stroke::new("WORLD"), Translation::text("World")),
            (Stroke::new("H-L"), Translation::text("another")),
        ]);

        let state = State::default();

        let (output, _) = translate(Stroke::new("H-L"), &dict, state);

        assert_eq!(output, Output::add_text(" another"));
    }

    #[test]
    fn test_translator_basic() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("H-L")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(translations, vec![Translation::text("Hello")]);
    }

    #[test]
    fn test_translator_multistroke() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("A"), Stroke::new("H-L")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::text("Wrong thing"), Translation::text("Hello")]
        );
    }

    #[test]
    fn test_translator_correction() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("H-L"), Stroke::new("A")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(translations, vec![Translation::text("He..llo")]);
    }

    #[test]
    fn test_translator_correction_with_history() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("WORLD"), Stroke::new("H-L"), Stroke::new("A")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::text("World"), Translation::text("He..llo")]
        );
    }

    #[test]
    fn test_translator_unknown_stroke() {
        let dict = setup_dict();
        let strokes = vec![Stroke::new("SKWR")];
        let translations = translate_strokes(&strokes, &dict);

        assert_eq!(
            translations,
            vec![Translation::UnknownStroke(Stroke::new("SKWR"))]
        );
    }

    #[test]
    fn test_translate_with_stroke_history() {
        let dict = setup_dict();
        let state = State::with_strokes(vec![Stroke::new("A")]);

        let (output, new_state) = translate(Stroke::new("H-L"), &dict, state);

        assert_eq!(output, Output::add_text(" Hello"));
        assert_eq!(
            new_state,
            State::with_strokes(vec![Stroke::new("A"), Stroke::new("H-L")])
        );
    }

    #[test]
    fn test_translate_no_stroke_history() {
        let dict = setup_dict();
        let state = State::default();

        let (output, new_state) = translate(Stroke::new("H-L"), &dict, state);

        assert_eq!(output, Output::add_text(" Hello"));
        assert_eq!(new_state, State::with_strokes(vec![Stroke::new("H-L")]));
    }

    #[test]
    fn test_translate_stroke_correction() {
        let dict = setup_dict();
        let state = State::with_strokes(vec![Stroke::new("H-L")]);

        let (output, new_state) = translate(Stroke::new("A"), &dict, state);

        assert_eq!(output, Output::replace(3, "..llo"));
        assert_eq!(
            new_state,
            State::with_strokes(vec![Stroke::new("H-L"), Stroke::new("A")])
        );
    }

    #[test]
    fn test_translation_diff_same() {
        let output = translation_diff(
            &vec![Translation::text("Hello"), Translation::text("Hi")],
            &vec![Translation::text("Hello"), Translation::text("Hi")],
        );

        assert_eq!(output, Output::add_text(""));
    }

    #[test]
    fn test_translation_diff_empty() {
        let output = translation_diff(&vec![], &vec![]);

        assert_eq!(output, Output::add_text(""));
    }

    #[test]
    fn test_translation_diff_simple_add() {
        let output = translation_diff(
            &vec![Translation::text("Hello")],
            &vec![Translation::text("Hello"), Translation::text("Hi")],
        );

        assert_eq!(output, Output::add_text(" Hi"));
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
    fn test_translate_unknown_stroke() {
        let dict = setup_dict();
        let state = State::default();

        let (output, new_state) = translate(Stroke::new("SKW-S"), &dict, state);

        assert_eq!(output, Output::add_text(" SKW-S"));
        assert_eq!(new_state, State::with_strokes(vec![Stroke::new("SKW-S")]));
    }

    #[test]
    fn test_word_change_first_letter() {
        assert_eq!(word_change_first_letter("hello".to_owned(), true), "Hello");
        assert_eq!(word_change_first_letter("".to_owned(), true), "");
        assert_eq!(word_change_first_letter("Hello".to_owned(), true), "Hello");
    }
}
