use plojo_core::{Command, Key, Modifier, SpecialKey, Stroke, Translator};
use plojo_standard::StandardTranslator;

/// Blackbox assert macro for better line number tracing
/// Expect that pressing stroke(s) causes a certain output
///
/// The stroke (or multiple strokes separated by '/') creates a command which is performed
///
/// The entire output (not just the added text) is matched against the total_output
macro_rules! b_expect {
    ($blackbox:expr, $strokes:expr, $expected:expr) => {
        $blackbox.lookup_and_dispatch($strokes);
        assert_eq!($blackbox.output, $expected);
    };
}

/// Blackbox assert macro for keys for better line number tracing
/// Expect that pressing stroke(s) causes certain key commands
/// Similar to b_expect
/// All of the keys produced are matched against total_keys
macro_rules! b_expect_keys {
    ($blackbox:expr, $strokes:expr, $expected:expr) => {
        $blackbox.lookup_and_dispatch($strokes);
        assert_eq!($blackbox.output_keys, $expected);
    };
}

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
        // allocate string with extra capacity for the brackets
        let json_str = String::with_capacity(raw_dict.len() + 2) + "{" + raw_dict + "}";
        Self::new_internal(json_str, false, false)
    }

    /// Creates a black box with stroke `AFPS` to retroactive add space. Inserts "S-P": "{^ ^}"
    /// into the dictionary for retroactive add space to work
    fn new_with_retroactive_add_space(raw_dict: &str) -> Self {
        // allocate string with extra capacity for the brackets and the S-P entry
        let json_str = String::with_capacity(raw_dict.len() + 18)
            + "{"
            + raw_dict
            + r#", "S-P": "{^ ^}""#
            + "}";
        Self::new_internal(json_str, true, false)
    }

    /// Creates a black box with stroke `AFPS` to retroactive add space. Inserts "S-P": "{^ ^}"
    /// into the dictionary for retroactive add space to work
    fn new_with_space_after(raw_dict: &str) -> Self {
        let json_str: String = "{".to_string() + raw_dict + "}";
        Self::new_internal(json_str, false, true)
    }

    fn new_internal(json_str: String, is_retro_add_space: bool, is_space_after: bool) -> Self {
        let translator = if is_retro_add_space {
            StandardTranslator::new(
                vec![json_str],
                vec![],
                vec![Stroke::new("AFPS")],
                Some(Stroke::new("S-P")),
                is_space_after,
            )
        } else {
            StandardTranslator::new(vec![json_str], vec![], vec![], None, is_space_after)
        }
        .expect("Unable to create translator");

        Self {
            translator,
            output: String::new(),
            output_keys: vec![],
        }
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
                            let output_len = self.output.chars().count();
                            self.output.truncate(output_len - backspace_num)
                        }

                        if !add_text.is_empty() {
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
                    Command::Shell(cmd, args) => {
                        panic!(
                            "Cannot handle shell commands. Command: {:?} with args: {:?}",
                            cmd, args
                        );
                    }
                    Command::TranslatorCommand(cmd) => {
                        self.translator.handle_command(cmd);
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
    b_expect!(b, "H-L/WORLD", " hello world");
}

#[test]
fn basic_undo() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "WORLD": "world"
        "#,
    );
    b_expect!(b, "H-L", " hello");
    b_expect!(b, "WORLD", " hello world");
    b_expect!(b, "TPHOT", " hello world TPHOT");
    b_expect!(b, "*", " hello world");
    b_expect!(b, "*", " hello");
    b_expect!(b, "*", "");
}

#[test]
fn basic_correction() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "H-L/WORLD": "hi"
        "#,
    );
    b_expect!(b, "H-L", " hello");
    b_expect!(b, "WORLD", " hi");
}

#[test]
fn double_space() {
    let mut b = Blackbox::new(
        r#"
            "S-P": "{^ ^}",
            "H-L": "hello"
        "#,
    );
    b_expect!(b, "H-L/S-P/S-P", " hello  ");
}

#[test]
fn first_punctuation() {
    let mut b = Blackbox::new(
        r#"
            "TP-PL": "{.}"
        "#,
    );
    b_expect!(b, "TP-PL", ".");
}

#[test]
fn first_attached() {
    let mut b = Blackbox::new(
        r#"
            "EURB": "{^ish}"
        "#,
    );
    b_expect!(b, "EURB", "ish");
    b_expect!(b, "EURB", "ishish");
}

#[test]
fn punctuation_with_attached() {
    let mut b = Blackbox::new(
        r#"
            "TP-PL": "{.}",
            "KR-GS": "{^~|\"}"
        "#,
    );
    b_expect!(b, "TP-PL/KR-GS", ".\"");
}

#[test]
fn unknown_with_attached() {
    let mut b = Blackbox::new(
        r#"
            "-D": "{^ed}"
        "#,
    );
    b_expect!(b, "STPW/-D", " STPWed");
}

#[test]
fn commands_correction() {
    let mut b = Blackbox::new(
        r#"
            "H-L": {"cmds": [{ "Keys": [{"Special": "UpArrow"}, []] }]},
            "H-L/WORLD": {"cmds": [{ "Keys": [{"Special": "UpArrow"}, []] }]},
            "H-L/WORLD/H-L": "hi"
        "#,
    );
    b_expect_keys!(b, "H-L", vec![(Key::Special(SpecialKey::UpArrow), vec![])]);
    b_expect_keys!(
        b,
        "WORLD",
        vec![(Key::Special(SpecialKey::UpArrow), vec![])]
    );
    b_expect!(b, "H-L", " hi");
}

#[test]
fn commands_undo() {
    let mut b = Blackbox::new(
        r#"
            "H-L": {"cmds": [{ "Keys": [{"Special": "UpArrow"}, []] }]},
            "H-L/WORLD": "hello",
            "TP": {"cmds": [{ "Keys": [{"Layout": "a"}, ["Meta"]] }]},
            "TPAO": "foo"
        "#,
    );
    b_expect_keys!(b, "H-L", vec![(Key::Special(SpecialKey::UpArrow), vec![])]);
    b_expect!(b, "WORLD", " hello");
    b_expect_keys!(
        b,
        "TP",
        vec![
            (Key::Special(SpecialKey::UpArrow), vec![]),
            (Key::Layout('a'), vec![Modifier::Meta]),
        ]
    );
    b_expect!(b, "TPAO", " hello foo");
    b_expect!(b, "*", " hello");
    b_expect!(b, "*", "");
}

#[test]
fn glued_strokes() {
    let mut b = Blackbox::new(
        r#"
            "TK*": "{&d}",
            "H-L": "hello"
        "#,
    );
    b_expect!(b, "H-L/TK*", " hello d");
    b_expect!(b, "TK*", " hello dd");
    b_expect!(b, "H-L", " hello dd hello");
}

#[test]
fn numbers_are_glued() {
    let mut b = Blackbox::new(
        r#"
            "TK*": "{&d}",
            "H-L": "hello"
        "#,
    );
    b_expect!(b, "TK*", " d");
    b_expect!(b, "123/1-8", " d12318");
    b_expect!(b, "H-L", " d12318 hello");
    b_expect!(b, "123", " d12318 hello 123");
}

#[test]
fn number_translation() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "{&hi}",
            "2-8D": "2800"
        "#,
    );
    b_expect!(b, "H-L", " hi");
    b_expect!(b, "12", " hi12");
    b_expect!(b, "2-8D", " hi122800");
}

#[test]
fn capitalize_word_after_command() {
    let mut b = Blackbox::new(
        r#"
            "KPA*": "{^}{-|}",
            "TKOUPB": {"cmds": [{ "Keys": [{"Special": "DownArrow"}, []] }]},
            "UP": {"cmds": [{ "Keys": [{"Special": "UpArrow"}, []] }]},
            "-T": "the"
        "#,
    );
    b_expect!(b, "-T", " the");
    b_expect_keys!(
        b,
        "KPA*/TKOUPB",
        vec![(Key::Special(SpecialKey::DownArrow), vec![])]
    );
    b_expect_keys!(
        b,
        "UP",
        vec![
            (Key::Special(SpecialKey::DownArrow), vec![]),
            (Key::Special(SpecialKey::UpArrow), vec![]),
        ]
    );
    b_expect!(b, "-T", " theThe");
}

#[test]
fn undo_suppress_space() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "TK-LS": "{^^}",
            "KPA*": "{^}{-|}",
            "TPAO": "foo"
        "#,
    );
    b_expect!(b, "H-L/TK-LS/KPA*/TPAO", " helloFoo");
    b_expect!(b, "*", " hello");
    b_expect!(b, "*", "");
}

#[test]
fn text_action_after_command() {
    let mut b = Blackbox::new(
        r#"
            "H-L": "hello",
            "TKOUPB": {
                "cmds": [{ "Keys": [{"Special": "DownArrow"}, []] }],
                "text_after": [
                    {
                        "Attached": {
                            "text": "",
                            "joined_next": true,
                            "do_orthography": false,
                            "carry_capitalization": false
                        }
                    },
                    { "StateAction": "ForceCapitalize" }
                ]
            },
            "TPAO": "foo"
        "#,
    );
    b_expect!(b, "H-L/TKOUPB/TPAO", " helloFoo");
}

#[test]
fn retrospective_actions() {
    let mut b = Blackbox::new_with_retroactive_add_space(
        r#"
            "H-L": "Hello World",
            "TKFPS": "{*!}",
            "KA*PD": "{*-|}",
            "TPAO": "foo",
            "TK-LS": "{^^}",
            "KPA": "{-|}"
        "#,
    );
    b_expect!(b, "H-L/TKFPS", " HelloWorld");
    b_expect!(b, "TPAO/KA*PD", " HelloWorld Foo");
    b_expect!(b, "TK-LS/TPAO/KPA", " HelloWorld Foofoo");
    b_expect!(b, "AFPS", " HelloWorld Foo foo");
}

#[test]
fn retrospective_add_space_breaks_up_translation() {
    let mut b = Blackbox::new_with_retroactive_add_space(
        r#"
            "H-L": "hello",
            "WORLD": "world",
            "H-L/WORLD": "Hello, world!",
            "H-L/WORLD/WORLD": "Big hello world"
        "#,
    );
    b_expect!(b, "H-L/WORLD", " Hello, world!");
    b_expect!(b, "WORLD", " Big hello world");
    b_expect!(b, "AFPS", " Hello, world! world");
}

#[test]
fn retrospective_add_space_glued() {
    let mut b = Blackbox::new_with_retroactive_add_space(
        r#"
            "H*": "{&h}",
            "*EU": "{&i}"
        "#,
    );
    b_expect!(b, "H*/*EU", " hi");
    b_expect!(b, "AFPS", " h i");
}

#[test]
fn basic_unicode() {
    let mut b = Blackbox::new(
        r#"
            "PH-RB": "—",
            "H-L": "hello",
            "PH-RB/H-L/H-L": "hello—"
        "#,
    );
    b_expect!(b, "PH-RB", " —");
    b_expect!(b, "H-L", " — hello");
    b_expect!(b, "H-L", " hello—");
}

#[test]
fn suppress_space_lowercases_word() {
    let mut b = Blackbox::new(
        r#"
            "TK-LS": "{^^}",
            "TP-PL": "{.}",
            "KPA": "{-|}",
            "H-L": "hello"
        "#,
    );
    b_expect!(b, "TP-PL/TK-LS/H-L", ".hello");
    b_expect!(b, "KPA/TK-LS/H-L", ".hellohello");
}

#[test]
fn force_cap_should_clear_suppress_space() {
    let mut b = Blackbox::new(
        r#"
            "TK-LS": "{^^}",
            "KPA*": "{^}{-|}",
            "KPA": "{}{-|}",
            "H-L": "hello"
        "#,
    );
    b_expect!(b, "TK-LS/KPA/H-L", " Hello");
    b_expect!(b, "KPA*/KPA/H-L", " Hello Hello");
}

#[test]
fn orthography_retro_add_space() {
    let mut b = Blackbox::new_with_retroactive_add_space(
        r#"
            "KAER": "carry",
            "-S": "{^s}"
        "#,
    );
    b_expect!(b, "KAER/-S", " carries");
    b_expect!(b, "AFPS", " carry s");
    b_expect!(b, "-S/AFPS", " carry s s");
}

#[test]
fn suffix_folding() {
    let mut b = Blackbox::new(
        r#"
            "-S": "{^s}",
            "-Z": "{^s}",
            "RAEUS": "race",
            "RAEUZ": "raise"
        "#,
    );
    b_expect!(b, "RAEUSZ", " races");
}

#[test]
fn suffix_folding_precedence() {
    let mut b = Blackbox::new(
        r#"
            "TPRAOEU": "Friday",
            "-S": "{^s}",
            "TPRAOEUS": "fries"
        "#,
    );
    b_expect!(b, "TPRAOEUS", " fries");
}

#[test]
fn space_after_suppress_space() {
    let mut b = Blackbox::new_with_space_after(
        r#"
            "H-L": "hello",
            "TK-LS": "{^^}"
        "#,
    );
    b_expect!(b, "H-L", "hello ");
    b_expect!(b, "TK-LS", "hello");
    b_expect!(b, "H-L", "hellohello ");
    b_expect!(b, "*", "hello");
    b_expect!(b, "*", "");
}

#[test]
fn space_after_suppress_space_before_command() {
    let mut b = Blackbox::new_with_space_after(
        r#"
            "R-R": {
                "cmds": [{ "Keys": [{"Special": "Return"}, []] }],
                "text_after": [
                    {
                        "Attached": {
                            "text": "",
                            "joined_next": true,
                            "do_orthography": false,
                            "carry_capitalization": false
                        }
                    },
                    { "StateAction": "ForceCapitalize" }
                ],
                "suppress_space_before": true
            },
            "H-L": "hello",
            "OBG": "okay"
        "#,
    );
    b_expect!(b, "H-L/R-R/OBG", "helloOkay ");
}

#[test]
fn space_after_duplicate_deletes() {
    let mut b = Blackbox::new_with_space_after(
        r#"
            "TW-B": {
                "cmds": [{ "Keys": [{"Special": "Tab"}, ["Meta"]] }],
                "suppress_space_before": true
            },
            "H-L": "hello"
        "#,
    );
    b_expect!(b, "H-L", "hello ");
    b_expect!(b, "TW-B", "hello");
    b_expect!(b, "TW-B", "hello");
    b_expect!(b, "TW-B", "hello");
}

#[test]
fn orthography_uppercase() {
    let mut b = Blackbox::new(
        r#"
            "KPA": "{-|}",
            "PWEUG": "big",
            "*ER": "{^er}"
        "#,
    );
    b_expect!(b, "KPA/PWEUG", " Big");
    b_expect!(b, "*ER", " Bigger");
}

#[test]
fn orthography_bypass_with_ortho_dict() {
    let mut b = Blackbox::new(
        r#"
            "KPA": "{-|}",
            "TKPWA*RPB": "garden",
            "-G": "{^ing}"
        "#,
    );
    b_expect!(b, "KPA/TKPWA*RPB", " Garden");
    b_expect!(b, "-G", " Gardening");
}

#[test]
fn suffix_folding_last_suffix() {
    // only the last key which is the suffix can be folded
    let mut b = Blackbox::new(
        r#"
            "TPHRAT": "flat",
            "-S": "{^s}"
        "#,
    );
    b_expect!(b, "TPHRATS", " flats");
    b_expect!(b, "STPHRATS", " flats STPHRATS");
}

#[test]
fn command_preserve_space() {
    // suppress space before should not do anything unless output is set to space after
    let mut b = Blackbox::new(
        r#"
            "R-R": {
                "cmds": [{ "Keys": [{"Special": "Return"}, []] }],
                "suppress_space_before": true
            },
            "S-P": "{^ ^}",
            "H-L": "hello"
        "#,
    );
    b_expect!(b, "H-L/S-P", " hello ");
    b_expect!(b, "R-R", " hello ");
}

#[test]
fn clear_prev_strokes_orthography() {
    // suppress space before should not do anything unless output is set to space after
    let mut b = Blackbox::new(
        r#"
            "R-R": {
                "cmds": [
                    { "Keys": [{"Special": "Return"}, []] },
                    { "TranslatorCommand": "clear_prev_strokes" }
                ],
                "text_after": [
                  {
                    "Attached": {
                      "text": "",
                      "joined_next": true,
                      "do_orthography": false,
                      "carry_capitalization": false
                    }
                  },
                  { "StateAction": "ForceCapitalize" }
                ],
                "suppress_space_before": true
            },
            "SKEL": "cancel",
            "-D": "{^ed}"
        "#,
    );
    b_expect!(b, "SKEL/-D", " canceled");
    b_expect!(b, "R-R/SKEL/-D", " canceledCanceled");
}
