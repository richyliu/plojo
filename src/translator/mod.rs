use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
/// A stroke can be a single stroke (ex: "H-L") or several strokes (ex:
/// "H-L/WORLD")
pub struct Stroke(String);

impl Stroke {
    pub fn new(stroke: &str) -> Self {
        Stroke(stroke.to_owned())
    }
    fn to_raw(self) -> String {
        self.0
    }
    pub fn append(self, other: Self) -> Self {
        Stroke(format!("{}/{}", self.to_raw(), other.to_raw()))
    }
}

#[derive(Debug, PartialEq)]
pub enum Output {
    Text(String),
}

impl Output {
    pub fn text(output: &str) -> Self {
        Self::Text(output.to_owned())
    }
}

pub struct Dictionary {
    strokes: HashMap<Stroke, Output>,
}

type DictEntries = Vec<(Stroke, Output)>;
type RawDictEntries = Vec<(String, String)>;

impl Dictionary {
    pub fn new(entries: DictEntries) -> Self {
        let mut hashmap = HashMap::new();
        for (stroke, output) in entries.into_iter() {
            hashmap.insert(stroke, output);
        }

        Dictionary { strokes: hashmap }
    }

    fn has(&self, stroke: &Stroke) -> bool {
        self.strokes.get(stroke).is_some()
    }

    fn get(&self, stroke: &Stroke) -> Option<&Output> {
        self.strokes.get(stroke)
    }
}

#[derive(Debug, PartialEq)]
pub struct State {
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

/// Translate needs: stroke, temporary state (previous strokes, next word
/// uppercase, etc), persistent state (dictionary(ies))
pub fn translate(stroke: Stroke, dict: &Dictionary, mut state: State) -> (&Output, State) {
    for i in (0..state.prev_strokes.len()).rev() {
        let mut prev_strokes_combined = stroke.clone();
        for s in &state.prev_strokes[0..i] {
            prev_strokes_combined = prev_strokes_combined.append(s.clone());
        }
        if let Some(output) = dict.get(&prev_strokes_combined) {
            state.add_stroke(stroke);
            return (output, state);
        }
    }
    if let Some(output) = dict.get(&stroke) {
        state.add_stroke(stroke);
        return (output, state);
    }
    panic!("Stroke {:?} not found in dictionary", stroke);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_has() {
        let dict = Dictionary::new(vec![
            (Stroke::new("H-L"), Output::text("Hello")),
            (Stroke::new("WORLD"), Output::text("World")),
        ]);

        assert!(dict.has(&Stroke::new("H-L")));
        assert!(!dict.has(&Stroke::new("TPHOG")));
    }

    #[test]
    fn test_translate_with_stroke_history() {
        let dict = Dictionary::new(vec![
            (Stroke::new("H-L"), Output::text("Hello")),
            (Stroke::new("WORLD"), Output::text("World")),
        ]);
        let state = State::with_strokes(vec![Stroke::new("TP")]);

        let (output, new_state) = translate(Stroke::new("H-L"), &dict, state);

        assert_eq!(output, &Output::text("Hello"));
        assert_eq!(
            new_state,
            State {
                prev_strokes: vec![Stroke::new("TP"), Stroke::new("H-L")]
            }
        );
    }
}
