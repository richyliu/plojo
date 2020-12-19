use crate::{Dict, DictName, Translation};
use serde_json::{self, Value};
use std::collections::HashMap;

/// Load the dictionaries from the filenames and the dictionary name. Returns the parsed dictionary
/// and its name
pub fn load_dictionaries(files: Vec<(String, DictName)>) -> Vec<(Dict, DictName)> {
    let mut dicts = Vec::with_capacity(files.len());

    for (raw, name) in files {
        let dict = parse_dictionary(&raw);
        dicts.push((dict, name));
    }

    dicts
}

/// Parses a dictionary into a map from output text to strokes.
///
/// Any dictionary entry that is an object can be looked up with the object as a JSON string
/// (omit spaces from the string)
///
/// The dictionary should be a string of a JSON object
///
/// # Panics
///
/// Panics if there is an error when parsing the dictionary.
fn parse_dictionary(raw_dict: &str) -> Dict {
    let mut dict: Dict = HashMap::new();

    let value: Value = serde_json::from_str(&raw_dict).expect("Dictionary is not JSON");
    let entries = value.as_object().expect("Dictionary is not a JSON object");

    // insert the JSON reversed (translation to stroke map)
    for (stroke, translation) in entries {
        // format non strings as raw JSON text
        let t: Translation = match translation {
            Value::String(translation_str) => translation_str.clone(),
            other => format!("{}", other),
        };

        // add the stroke to the other strokes for this translation or make a new vec
        if let Some(v) = dict.get_mut(&t) {
            v.push(stroke.clone());
        } else {
            dict.insert(t, vec![stroke.clone()]);
        }
    }

    dict
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dictionary_basic() {
        let dict = parse_dictionary(
            r#"
            {
                "H-L": "hello",
                "HEL/HRO": "hello",
                "H*EL": "hello",
                "WORLD": "world",
                "STPR*EU": {"cmds": [{ "Shell": ["open", ["-a", "Safari"]] }]}
            }
            "#,
        );

        assert!(dict.get("world").is_some());
        assert!(dict.get("this does not exist").is_none());
        assert_eq!(dict.get("hello").unwrap().len(), 3);
        assert_eq!(
            dict.get(r#"{"cmds":[{"Shell":["open",["-a","Safari"]]}]}"#)
                .unwrap(),
            &vec!["STPR*EU".to_string()]
        );
    }
}
