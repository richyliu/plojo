use crate::Translation;
use std::collections::HashMap;
use std::error::Error;
use std::iter::FromIterator;
use translator::Stroke;

mod load;
mod translate;

type DictEntry = (Stroke, Vec<Translation>);

#[derive(Debug, PartialEq)]
pub struct Dictionary {
    strokes: HashMap<Stroke, Vec<Translation>>,
}

impl Dictionary {
    /// Create a new dictionary from raw JSON strings. Each string represents a dictionary, with
    /// each dictionaries being able to overwrite any dictionary entry before it
    pub fn new(raw_dicts: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let mut entries = vec![];
        for raw_dict in raw_dicts {
            entries.append(&mut load::load_dicts(&raw_dict)?);
        }

        Ok(entries.into_iter().collect())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Text, Translation};

    #[test]
    fn dictionary_overwrite() {
        let raw_dict1 = r#"
            {
                "H-L": "hello",
                "WORLD": "world"
            }
        "#
        .to_string();
        let raw_dict2 = r#"
            {
                "WORLD": "something else"
            }
        "#
        .to_string();

        let dict = Dictionary::new(vec![raw_dict1, raw_dict2]).unwrap();
        assert_eq!(
            dict.lookup(&[Stroke::new("WORLD")]).unwrap(),
            vec![Translation::Text(Text::Lit("something else".to_string()))]
        );
    }
}
