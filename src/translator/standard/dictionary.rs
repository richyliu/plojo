use crate::translator::standard::Translation;
use crate::Stroke;
use std::collections::HashMap;
use std::error::Error;
use std::iter::FromIterator;

mod load;
mod translate;

type DictEntry = (Stroke, Vec<Translation>);

#[derive(Debug, PartialEq)]
pub struct Dictionary {
    strokes: HashMap<Stroke, Vec<Translation>>,
}

impl Dictionary {
    pub fn new(raw: &str) -> Result<Self, Box<dyn Error>> {
        load::load(raw).map_err(|e| e.into())
    }

    pub(super) fn lookup(&self, strokes: &[Stroke]) -> Option<Vec<Translation>> {
        // combine strokes with a `/` between them
        let mut combined = strokes
            .into_iter()
            .map(|s| s.clone().to_raw())
            .fold(String::new(), |acc, s| acc + &s + "/");
        // remove trailing `/`
        combined.pop();

        self.strokes.get(&Stroke::new(&combined)).cloned()
    }

    pub(super) fn translate(&self, strokes: &Vec<Stroke>) -> Vec<Translation> {
        translate::translate_strokes(self, strokes)
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
