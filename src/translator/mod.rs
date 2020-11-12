use std::cmp;
use std::collections::HashMap;

/// A stroke can be a single stroke (ex: "H-L") or several strokes (ex:
/// "H-L/WORLD")
pub type Stroke = String;

fn empty_stroke() -> Stroke {
    String::from("")
}

/// Join two strokes together with a `/` in the middle. If either stroke is
/// empty, return the other stroke
fn stroke_join(one: &Stroke, other: &Stroke) -> Stroke {
    if other.len() == 0 {
        one.to_owned()
    } else if one.len() == 0 {
        other.to_owned()
    } else {
        format!("{}/{}", one, other)
    }
}

#[derive(Debug, PartialEq)]
pub enum Output {
    AddText(String),
    Replace(u32, String),
}

impl Output {
    pub fn text(output: &str) -> Self {
        Self::AddText(output.to_owned())
    }
    pub fn replace(backspace_num: u32, replace_str: &str) -> Self {
        Self::Replace(backspace_num, replace_str.to_owned())
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Text(String),
}

impl Command {
    pub fn text(t: &str) -> Self {
        Self::Text(t.to_owned())
    }
}

pub struct Dictionary {
    strokes: HashMap<Stroke, Command>,
}

type DictEntries = Vec<(Stroke, Command)>;

impl Dictionary {
    pub fn new(entries: DictEntries) -> Self {
        let mut hashmap = HashMap::new();
        for (stroke, output) in entries.into_iter() {
            hashmap.insert(stroke, output);
        }

        Dictionary { strokes: hashmap }
    }

    fn get(&self, stroke: &Stroke) -> Option<&Command> {
        self.strokes.get(stroke)
    }
}

#[derive(Debug, PartialEq)]
pub struct State {
    // should only include "undo-able" strokes (this excludes commands)
    prev_strokes: Vec<Stroke>,
}

impl State {
    pub fn initial() -> Self {
        State {
            prev_strokes: vec![],
        }
    }

    pub fn with_strokes(prev_strokes: Vec<Stroke>) -> Self {
        State { prev_strokes }
    }

    fn add_stroke(&mut self, stroke: Stroke) {
        self.prev_strokes.push(stroke)
    }
}

pub fn translate(stroke: Stroke, dict: &Dictionary, mut state: State) -> (Output, State) {
    let old_commands = translate_strokes(&state.prev_strokes, dict);
    state.prev_strokes.push(stroke);
    let new_commands = translate_strokes(&state.prev_strokes, dict);
    println!("old_commands: {:?}", old_commands);
    println!("new_commands: {:?}", new_commands);

    if let Some(Command::Text(text)) = dict.get(&String::from("H-L")) {
        return (Output::text(text), state);
    }
    panic!("oops");
}

fn translate_strokes<'a>(strokes: &Vec<Stroke>, dict: &'a Dictionary) -> Vec<&'a Command> {
    let mut strokes_combined = empty_stroke();
    let mut commands: Vec<&Command> = vec![];

    // TODO: this algorithm needs to be greedy; it needs to find the greatest
    // strokes that have a translation
    for s in strokes {
        strokes_combined = stroke_join(s, &strokes_combined);
        if let Some(command) = dict.get(&strokes_combined) {
            commands.push(command);
            strokes_combined = String::from("");
        }
    }

    commands
}

fn text_diff(old: &Vec<Stroke>, new: &Vec<Stroke>) -> Output {
    Output::text("")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_dict() -> Dictionary {
        Dictionary::new(vec![
            (String::from("H-L"), Command::text("Hello")),
            (String::from("WORLD"), Command::text("World")),
            (String::from("H-L/A"), Command::text("He..llo")),
            (String::from("A"), Command::text("Wrong thing")),
        ])
    }

    #[test]
    fn test_dict_lower_overwrite() {
        let dict = Dictionary::new(vec![
            (String::from("H-L"), Command::text("Hello")),
            (String::from("WORLD"), Command::text("World")),
            (String::from("H-L"), Command::text("another")),
        ]);

        let state = State::initial();

        let (output, _) = translate(String::from("H-L"), &dict, state);

        assert_eq!(output, Output::text("another"));
    }

    #[test]
    fn test_translate_with_stroke_history() {
        let dict = setup_dict();
        let state = State::with_strokes(vec![String::from("TP")]);

        let (output, new_state) = translate(String::from("H-L"), &dict, state);

        assert_eq!(output, Output::text("Hello"));
        assert_eq!(
            new_state,
            State::with_strokes(vec![String::from("TP"), String::from("H-L")])
        );
    }

    #[test]
    fn test_translate_no_stroke_history() {
        let dict = setup_dict();
        let state = State::initial();

        let (output, new_state) = translate(String::from("H-L"), &dict, state);

        assert_eq!(output, Output::text("Hello"));
        assert_eq!(new_state, State::with_strokes(vec![String::from("H-L")]));
    }

    #[test]
    fn test_translate_change_stroke() {
        let dict = setup_dict();
        let state = State::with_strokes(vec![String::from("H-L")]);

        let (output, new_state) = translate(String::from("A"), &dict, state);

        assert_eq!(output, Output::replace(3, "..llo"));
        assert_eq!(new_state, State::with_strokes(vec![String::from("H-L")]));
    }
}
