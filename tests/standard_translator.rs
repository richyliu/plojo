use plojo::{
    parse_command, ControllerAction, StandardTranslator, StandardTranslatorConfig, Stroke,
    Translator,
};

struct Blackbox {
    translator: StandardTranslator,
    output: String,
}

impl Blackbox {
    fn new(raw_dict: &str) -> Self {
        let translator =
            StandardTranslator::new(StandardTranslatorConfig::new(raw_dict.to_string(), vec![]))
                .expect("Unable to create translator");

        Self {
            translator,
            output: String::new(),
        }
    }

    fn expect(&mut self, next_stroke: &str, total_output: &str) {
        let stroke = Stroke::new(next_stroke);
        if !stroke.is_valid() {
            panic!("{:?} is not a valid stroke", stroke);
        }

        let command = if stroke.is_undo() {
            self.translator.undo()
        } else {
            self.translator.translate(stroke)
        };

        let actions = parse_command(command);
        for action in actions {
            match action {
                ControllerAction::TypeWithDelay(new_str, _) => self.output.push_str(&new_str),
                ControllerAction::BackspaceWithDelay(backspace_num, _) => {
                    self.output.truncate(self.output.len() - backspace_num)
                }
            }
        }

        assert_eq!(self.output, total_output);
    }
}

#[test]
fn test_basic_translation() {
    let mut b = Blackbox::new(
        r#"
            {
                "H-L": "hello",
                "WORLD": "world"
            }
        "#,
    );

    b.expect("H-L", " hello");
    b.expect("WORLD", " hello world");
}

#[test]
fn test_undo() {
    let mut b = Blackbox::new(
        r#"
            {
                "H-L": "hello",
                "WORLD": "world"
            }
        "#,
    );

    b.expect("H-L", " hello");
    b.expect("WORLD", " hello world");
    b.expect("*", " hello");
    b.expect("*", "");
}
