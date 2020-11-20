use crate::translator::{DictEntries, DictEntry};
use crate::{Stroke, Translation};
use std::collections::HashMap;
use std::iter::FromIterator;

pub struct Dictionary {
    strokes: HashMap<Stroke, Vec<Translation>>,
}

impl Dictionary {
    pub fn new(entries: DictEntries) -> Self {
        Self::from_iter(entries.into_iter())
    }

    pub fn lookup(&self, strokes: &[Stroke]) -> Option<Vec<Translation>> {
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
