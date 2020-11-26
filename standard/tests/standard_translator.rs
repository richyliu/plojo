use standard::{Config as StandardTranslatorConfig, StandardTranslator};
use translator::{Command, ExternalCommand, Stroke, Translator};

/// Black box for testing the entire translator
struct Blackbox {
    output: String,
    translator: StandardTranslator,
}

impl Blackbox {
    /// Create a new black box from with dictionary definitions
    ///
    /// raw_dict should be in a JSON string format. The outermost brackets should be omitted
    fn new(raw_dict: &str) -> Self {
        let json_str: String = "{".to_string() + raw_dict + "}";
        let translator =
            StandardTranslator::new(StandardTranslatorConfig::new(vec![json_str], vec![]))
                .expect("Unable to create translator");

        Self {
            translator,
            output: String::new(),
        }
    }

    /// Expect that pressing stroke(s) causes a certain output
    ///
    /// The stroke (or multiple strokes separated by '/') creates a command which is performed
    ///
    /// The entire output (not just the added text) is matched against the total_output
    fn expect(&mut self, strokes: &str, total_output: &str) {
        for s in strokes.split('/') {
            let stroke = Stroke::new(s);
            if !stroke.is_valid() {
                panic!("{:?} is not a valid stroke", stroke);
            }

            let command = if stroke.is_undo() {
                self.translator.undo()
            } else {
                self.translator.translate(stroke)
            };

            match command {
                Command::Internal(_) => {}
                Command::External(external_command) => match external_command {
                    ExternalCommand::Replace(backspace_num, add_text) => {
                        if backspace_num > 0 {
                            self.output.truncate(self.output.len() - backspace_num)
                        }

                        if add_text.len() > 0 {
                            self.output.push_str(&add_text);
                        }
                    }
                    ExternalCommand::PrintHello => {
                        println!("Hello!");
                    }
                },
                Command::NoOp => {}
            }
        }

        assert_eq!(self.output, total_output);
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

// see plover blackbox tests: https://github.com/openstenoproject/plover/blob/master/test/test_blackbox.py
