use plojo_core::{Command, Key, Modifier, SpecialKey, Stroke, Translator};
use plojo_standard::{Config as StandardTranslatorConfig, StandardTranslator};

/// Black box for testing the entire translator
struct Blackbox {
    output: String,
    translator: StandardTranslator,
    output_keys: Vec<(Key, Vec<Modifier>)>,
}

impl Blackbox {
    /// Create a new black box from with dictionary definitions
    ///
    /// raw_dict should be in a JSON string format. The outermost brackets should be omitted
    fn new(raw_dict: &str) -> Self {
        let json_str: String = "{".to_string() + raw_dict + "}";
        let translator =
            StandardTranslator::new(StandardTranslatorConfig::new().with_raw_dicts(vec![json_str]))
                .expect("Unable to create translator");

        Self {
            translator,
            output: String::new(),
            output_keys: vec![],
        }
    }

    /// Expect that pressing stroke(s) causes a certain output
    ///
    /// The stroke (or multiple strokes separated by '/') creates a command which is performed
    ///
    /// The entire output (not just the added text) is matched against the total_output
    fn expect(&mut self, strokes: &str, total_output: &str) {
        self.lookup_and_dispatch(strokes);
        assert_eq!(self.output, total_output);
    }

    /// Expect that pressing stroke(s) causes certain key commands
    /// Similar to expect
    /// All of the keys produced are matched against total_keys
    fn expect_keys(&mut self, strokes: &str, total_keys: Vec<(Key, Vec<Modifier>)>) {
        self.lookup_and_dispatch(strokes);
        assert_eq!(self.output_keys, total_keys);
    }

    fn lookup_and_dispatch(&mut self, strokes: &str) {
        for s in strokes.split('/') {
            let stroke = Stroke::new(s);
            if !stroke.is_valid() {
                panic!("{:?} is not a valid stroke", stroke);
            }

            let commands = if stroke.is_undo() {
                self.translator.undo()
            } else {
                self.translator.translate(stroke)
            };

            for command in commands {
                match command {
                    Command::Replace(backspace_num, add_text) => {
                        if backspace_num > 0 {
                            self.output.truncate(self.output.len() - backspace_num)
                        }

                        if add_text.len() > 0 {
                            self.output.push_str(&add_text);
                        }
                    }
                    Command::PrintHello => {
                        panic!("Not expecting PrintHello to be outputted from the blackbox");
                    }
                    Command::NoOp => {}
                    Command::Keys(key, modifiers) => {
                        self.output_keys.push((key, modifiers));
                    }
                    Command::Raw(code) => {
                        panic!("Cannot handle raw keycodes. Raw key code: {}", code);
                    }
                }
            }
        }
    }
}

#[test]
fn basic_translation() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "WORLD": "world"
        "#,
    );
    b.expect("H-L/WORLD", " hello world");
}

#[test]
fn basic_undo() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "WORLD": "world"
        "#,
    );
    b.expect("H-L", " hello");
    b.expect("WORLD", " hello world");
    b.expect("*", " hello");
    b.expect("*", "");
}

#[test]
fn basic_correction() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "H-L/WORLD": "hi"
        "#,
    );
    b.expect("H-L", " hello");
    b.expect("WORLD", " hi");
}

#[test]
fn double_space() {
    let mut b = Blackbox::new(
        r#"
            "S-P": "{^ ^}",
            "H-L": "hello"
        "#,
    );
    b.expect("H-L/S-P/S-P", " hello  ");
}

#[test]
fn first_punctuation() {
    let mut b = Blackbox::new(
        r#"
            "TP-PL": "{.}"
        "#,
    );
    b.expect("TP-PL", ".");
}

#[test]
fn first_attached() {
    let mut b = Blackbox::new(
        r#"
            "EURB": "{^ish}"
        "#,
    );
    b.expect("EURB", "ish");
    b.expect("EURB", "ishish");
}

#[test]
fn punctuation_with_attached() {
    let mut b = Blackbox::new(
        r#"
            "TP-PL": "{.}",
            "KR-GS": "{^~|\"}"
        "#,
    );
    b.expect("TP-PL/KR-GS", ".\"");
}

#[test]
fn unknown_with_attached() {
    let mut b = Blackbox::new(
        r#"
            "-D": "{^ed}"
        "#,
    );
    b.expect("STPW/-D", " STPWed");
}

#[test]
fn suppress_space_should_lowercase() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "TK-LS": "{^^}",
            "KPA*": "{^}{-|}"
        "#,
    );
    b.expect("H-L/KPA*/TK-LS/H-L", " hellohello");
}

#[test]
fn commands_correction() {
    let mut b = Blackbox::new(
        r#"
            "H-L": [{ "Keys": [{"Special": "UpArrow"}, []] }],
            "H-L/WORLD": [{ "Keys": [{"Special": "UpArrow"}, []] }],
            "H-L/WORLD/H-L": "hi"
        "#,
    );
    b.expect_keys("H-L", vec![(Key::Special(SpecialKey::UpArrow), vec![])]);
    b.expect_keys("WORLD", vec![(Key::Special(SpecialKey::UpArrow), vec![])]);
    b.expect("H-L", " hi");
}

#[test]
fn commands_undo() {
    let mut b = Blackbox::new(
        r#"
            "H-L": [{ "Keys": [{"Special": "UpArrow"}, []] }],
            "H-L/WORLD": "hello",
            "TP": [{ "Keys": [{"Layout": "a"}, ["Meta"]] }]
        "#,
    );
    b.expect_keys("H-L", vec![(Key::Special(SpecialKey::UpArrow), vec![])]);
    b.expect("WORLD", " hello");
    b.expect_keys(
        "TP",
        vec![
            (Key::Special(SpecialKey::UpArrow), vec![]),
            (Key::Layout('a'), vec![Modifier::Meta]),
        ],
    );
    b.expect("*", " hello");
    b.expect("*", "");
    b.expect_keys(
        "*",
        vec![
            (Key::Special(SpecialKey::UpArrow), vec![]),
            (Key::Layout('a'), vec![Modifier::Meta]),
        ],
    );
}

#[test]
fn glued_strokes() {
    let mut b = Blackbox::new(
        r#"
            "TK*": "{&d}",
            "H-L": "hello"
        "#,
    );
    b.expect("H-L/TK*", " hello d");
    b.expect("TK*", " hello dd");
    b.expect("H-L", " hello dd hello");
}

#[test]
fn numbers_are_glued() {
    let mut b = Blackbox::new(
        r#"
            "TK*": "{&d}",
            "H-L": "hello"
        "#,
    );
    b.expect("TK*", " d");
    b.expect("123/1-8", " d12318");
    b.expect("H-L", " d12318 hello");
    b.expect("123", " d12318 hello 123");
}

#[test]
fn number_translation() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "{&hi}",
            "2-8D": "2800"
        "#,
    );
    b.expect("H-L", " hi");
    b.expect("12", " hi12");
    b.expect("2-8D", " hi122800");
}
