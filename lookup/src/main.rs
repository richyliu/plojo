use std::{collections::HashMap, env, fs, path::Path};
use toml::Value;

mod load;

type Stroke = String;
type Translation = String;
type Dict = HashMap<Translation, Vec<Stroke>>;
type DictName = String;

fn main() {
    let query = get_query();
    // assume config file with list of dictionaries is at ~/.plojo/config.toml
    let config_base = Path::new(&dirs::home_dir().unwrap()).join(".plojo");
    let raw_config = fs::read_to_string(config_base.join("config.toml"))
        .expect("unable to read config.toml file");
    let value = raw_config.parse::<Value>().unwrap();
    // assume config file has dicts key with list of strings which are the dictionary names
    let dicts: Vec<(String, String)> = value["dicts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|val| val.as_str().unwrap())
        .map(|name| {
            // assume dictionaries are in ~/.plojo/dicts/
            let file_name = config_base.join("dicts").join(name);
            let raw = fs::read_to_string(file_name).expect("Unable to read dictionary");
            (raw, name.to_string())
        })
        .collect::<Vec<_>>();
    let dicts = load::load_dictionaries(dicts);

    println!("Searching for: {}", query);

    let matches = lookup(&dicts, query);
    if matches.is_empty() {
        println!("Not found");
    } else {
        // count total number of matches for each dictionary matched
        let num_matches = matches.iter().fold(0, |acc, (m, _)| acc + m.len());
        if num_matches == 1 {
            println!("1 match found");
        } else {
            println!("{} matches found", num_matches);
        }
        println!("{}", format_lookup(&matches));
    }
}

fn get_query() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("You must pass in a search string as the argument");
    }
    args[1].to_string()
}

/// Look up a given translation in the dictionaries.
///
/// The translation should be the literal string in the dictionary or a string representation of
/// the JSON object in the dictionary.
fn lookup(dicts: &[(Dict, DictName)], translation: Translation) -> Vec<(&Vec<Stroke>, &DictName)> {
    let mut strokes = vec![];
    for (d, dict_name) in dicts {
        if let Some(s) = d.get(&translation) {
            strokes.push((s, dict_name));
        }
    }
    strokes
}

/// Format the matches as a string of the dictionary name and the matched strokes
fn format_lookup(matches: &[(&Vec<Stroke>, &DictName)]) -> String {
    let mut all_str = String::new();

    for (m, dict_name) in matches {
        let mut s: String = "\nFile: ".to_string() + dict_name + "\n";
        for stroke in *m {
            s.push_str(stroke);
            s.push_str("\n");
        }
        all_str.push_str(&s);
    }

    all_str
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testing_dict() -> Vec<(Dict, DictName)> {
        vec![
            (
                [
                    (
                        "hello".to_string(),
                        vec![
                            "H-L".to_string(),
                            "H*EL".to_string(),
                            "HEL/HRO".to_string(),
                            "HO*EL".to_string(),
                        ],
                    ),
                    (
                        "world".to_string(),
                        vec![
                            "WORLD".to_string(),
                            "WORLTD".to_string(),
                            "WORL".to_string(),
                        ],
                    ),
                ]
                .iter()
                .cloned()
                .collect::<Dict>(),
                "default.json".to_string(),
            ),
            (
                [(
                    "world".to_string(),
                    vec!["WORLD".to_string(), "WORLD/WORLD".to_string()],
                )]
                .iter()
                .cloned()
                .collect::<Dict>(),
                "secondary.json".to_string(),
            ),
        ]
    }

    #[test]
    fn lookup_basic() {
        let dicts = testing_dict();
        assert_eq!(
            lookup(&dicts, "hello".to_string()),
            vec![(
                &vec![
                    "H-L".to_string(),
                    "H*EL".to_string(),
                    "HEL/HRO".to_string(),
                    "HO*EL".to_string(),
                ],
                &"default.json".to_string()
            )]
        );
        assert_eq!(
            lookup(&dicts, "world".to_string()),
            vec![
                (
                    &vec![
                        "WORLD".to_string(),
                        "WORLTD".to_string(),
                        "WORL".to_string(),
                    ],
                    &"default.json".to_string()
                ),
                (
                    &vec!["WORLD".to_string(), "WORLD/WORLD".to_string()],
                    &"secondary.json".to_string()
                )
            ]
        );
        // search should be case sensitive
        assert_eq!(lookup(&dicts, "World".to_string()), vec![]);
    }

    #[test]
    fn format_basic() {
        assert_eq!(
            format_lookup(&vec![
                (
                    &vec!["H-L".to_string(), "H*EL".to_string()],
                    &"default.json".to_string(),
                ),
                (&vec!["HEL/HRO".to_string()], &"secondary.json".to_string()),
            ]),
            r#"
File: default.json
H-L
H*EL

File: secondary.json
HEL/HRO
"#
        )
    }
}
