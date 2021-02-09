#[macro_use]
extern crate lazy_static;

use clap::{App, Arg};
use plojo_core::{Command, Key, Modifier, SpecialKey};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::fs;

fn main() {
    let matches = App::new("Plover dictionary converter")
        .version("0.1.0")
        .about(
            "Converts commands in the plover dictionary format to one accepted by plojo.
Only commands in the following format can be converted:

{^}{#shift_l(Alt_L(tab))}{^}{-|}

The {^} at the front is optional and the {^}{-|} at the end can be {^} and is
optional. There can only be one key + modifiers in the keyboard shortcut.
Modifiers should precede the key as shown in the example. Outputs converted
dictionary to stdout.",
        )
        .arg(
            Arg::with_name("FILE")
                .required(true)
                .help("Input dictionary file to convert")
                .takes_value(true),
        )
        .get_matches();

    let filename = matches.value_of("FILE").unwrap();
    let contents = fs::read_to_string(filename).expect("unable to read file");

    let mut value: Value = serde_json::from_str(&contents).expect("unable to parse JSON");
    convert(&mut value);

    println!("{}", serialize(&value));
}

/// Serialize a JSON object with one entry on each line.
///
/// Object keys are serialized in alphabetical order
fn serialize(dict: &Value) -> String {
    if let Value::Object(map) = dict {
        if map.is_empty() {
            return "{}".to_string();
        }

        let mut result = "{".to_string();

        for (stroke, translation) in map.iter() {
            result += "\n\"";
            result += stroke;
            result += "\": ";
            result += &translation.to_string();
            result += ",";
        }

        // remove trailing comma from the last entry
        result.pop();
        result += "\n}";

        result
    } else {
        panic!("top level serialize value should be an object");
    }
}

fn convert(value: &mut Value) {
    let object_entries = value
        .as_object_mut()
        .expect("dictionary top level should be an object");

    for (stroke, translation) in object_entries.iter_mut() {
        match translation {
            Value::String(original) => {
                *translation = if original == "{#}" {
                    // ignore a "do nothing" stroke
                    continue;
                } else if original.contains("{#") {
                    // must convert plover shortcut format if it exists
                    match convert_keyboard_shortcut(original) {
                        Ok(converted) => converted,
                        Err(e) => {
                            eprintln!(
                                r#"[WARN]: Could not convert "{}": "{}" because of {:?}"#,
                                stroke, original, e
                            );
                            // could not be parsed; ignore
                            continue;
                        }
                    }
                } else {
                    // ignore non command strokes
                    continue;
                };
            }
            _t => {
                eprintln!("[ERR]: key '{}' has value {}", stroke, _t);
                panic!("plover dictionary should only contain string values");
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum ConversionError {
    InvalidFormat,
    InvalidKeyboardShortcut,
    UnbalancedParens,
    UnknownModifier(String),
    UnknownKey(String),
}

#[derive(Serialize)]
struct Cmd {
    cmds: Vec<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_after: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    suppress_space_before: bool,
}

/// Convert a basic keyboard shortcut string into a command that can be interpreted by plojo.
///
/// This is the basic format: `{^}{#Shift_L(Alt_L(a))}{^}{-|}`
/// Where the `{^}` in the beginning is optional and the ending `{^}` and `{-|}` are optional
///
/// The keyboard shortcut in the middle follows the pattern `{#..}`. There must be only one
/// shortcut key (no spaces).
///
/// The modifier keys are translated into the plojo format in the order they appear.
///
/// The text-after and suppress_space_before fields will not be serialized unless they are
/// necessary.
fn convert_keyboard_shortcut(s: &str) -> Result<Value, ConversionError> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"^((?:\{\^\})?)\{#([^\} ]+)\}((?:\{\^\}(?:\{-\|\})?)?)$"#).unwrap();
    }

    if let Some(c) = RE.captures(s) {
        let cmd = parse_key_combo(&c[2])?;
        let text_after = match &c[3] {
            "{^}{-|}" => Some(c[3].to_owned()),
            "{^}" => Some(c[3].to_owned()),
            "" => None,
            _ => unreachable!(),
        };
        let suppress_space_before = match &c[1] {
            "{^}" => true,
            "" => false,
            _ => unreachable!(),
        };

        let cmd = Cmd {
            cmds: vec![cmd],
            text_after,
            suppress_space_before,
        };

        Ok(serde_json::to_value(cmd).unwrap())
    } else {
        Err(ConversionError::InvalidFormat)
    }
}

/// Parses a single plover keyboard shortcut string into a plojo recognizable command
///
/// See plover documentation for details
/// https://github.com/openstenoproject/plover/wiki/Dictionary-Format#keyboard-shortcuts
///
/// This only accepts a single key + modifiers. Multiple keys do not work (there should not be
/// spaces)
fn parse_key_combo(s: &str) -> Result<Command, ConversionError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^((?:[a-z_]+\()*)([a-z0-9_]+)(\)*)$"#).unwrap();
    }

    let s = s.to_lowercase();

    if let Some(c) = RE.captures(&s) {
        let num_modifiers = c[3].len();
        let mut modifiers_str: Vec<&str> = c[1].split('(').collect();
        // remove last item created by trailing '(' from the regex
        assert_eq!(modifiers_str.pop().unwrap(), "");

        if modifiers_str.len() != num_modifiers {
            return Err(ConversionError::UnbalancedParens);
        }

        let mut modifiers = Vec::with_capacity(modifiers_str.len());
        for m in modifiers_str {
            modifiers.push(parse_plover_modifier(m)?);
        }

        let key = parse_plover_key(&c[2])?;

        Ok(Command::Keys(key, modifiers))
    } else {
        Err(ConversionError::InvalidKeyboardShortcut)
    }
}

/// Parses a lowercased plover modifier into a plojo modifier (parsable into a command)
fn parse_plover_modifier(m: &str) -> Result<Modifier, ConversionError> {
    match m {
        "shift_l" | "shift_r" | "shift" => Ok(Modifier::Shift),
        "control_l" | "control_r" | "control" => Ok(Modifier::Control),
        "alt_l" | "alt_r" | "alt" => Ok(Modifier::Alt),
        "option" => Ok(Modifier::Option),
        "super_l" | "super_r" | "super" | "windows" | "command" => Ok(Modifier::Meta),
        _m => Err(ConversionError::UnknownModifier(_m.to_owned())),
    }
}

/// Parses a lowercased plover key into a plojo key (parsable into a command)
fn parse_plover_key(k: &str) -> Result<Key, ConversionError> {
    match k {
        "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o"
        | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" | "0" | "1" | "2"
        | "3" | "4" | "5" | "6" | "7" | "8" | "9" => Ok(Key::Layout(k.chars().next().unwrap())),
        "backspace" => Ok(Key::Special(SpecialKey::Backspace)),
        "caps_lock" => Ok(Key::Special(SpecialKey::CapsLock)),
        "delete" => Ok(Key::Special(SpecialKey::Delete)),
        "end" => Ok(Key::Special(SpecialKey::End)),
        "escape" => Ok(Key::Special(SpecialKey::Escape)),
        "home" => Ok(Key::Special(SpecialKey::Home)),
        "page_down" => Ok(Key::Special(SpecialKey::PageDown)),
        "page_up" => Ok(Key::Special(SpecialKey::PageUp)),
        "return" => Ok(Key::Special(SpecialKey::Return)),
        "space" => Ok(Key::Special(SpecialKey::Space)),
        "tab" => Ok(Key::Special(SpecialKey::Tab)),
        "down" => Ok(Key::Special(SpecialKey::DownArrow)),
        "left" => Ok(Key::Special(SpecialKey::LeftArrow)),
        "right" => Ok(Key::Special(SpecialKey::RightArrow)),
        "up" => Ok(Key::Special(SpecialKey::UpArrow)),
        "f1" => Ok(Key::Special(SpecialKey::F1)),
        "f2" => Ok(Key::Special(SpecialKey::F2)),
        "f3" => Ok(Key::Special(SpecialKey::F3)),
        "f4" => Ok(Key::Special(SpecialKey::F4)),
        "f5" => Ok(Key::Special(SpecialKey::F5)),
        "f6" => Ok(Key::Special(SpecialKey::F6)),
        "f7" => Ok(Key::Special(SpecialKey::F7)),
        "f8" => Ok(Key::Special(SpecialKey::F8)),
        "f9" => Ok(Key::Special(SpecialKey::F9)),
        "f10" => Ok(Key::Special(SpecialKey::F10)),
        "f11" => Ok(Key::Special(SpecialKey::F11)),
        "f12" => Ok(Key::Special(SpecialKey::F12)),
        // copied from plover/key_combo.py
        "aacute" => Ok(Key::Layout('á')),
        "acircumflex" => Ok(Key::Layout('â')),
        "acute" => Ok(Key::Layout('´')),
        "adiaeresis" => Ok(Key::Layout('ä')),
        "ae" => Ok(Key::Layout('æ')),
        "agrave" => Ok(Key::Layout('à')),
        "ampersand" => Ok(Key::Layout('&')),
        "apostrophe" => Ok(Key::Layout('\'')),
        "aring" => Ok(Key::Layout('å')),
        "asciicircum" => Ok(Key::Layout('^')),
        "asciitilde" => Ok(Key::Layout('~')),
        "asterisk" => Ok(Key::Layout('*')),
        "at" => Ok(Key::Layout('@')),
        "atilde" => Ok(Key::Layout('ã')),
        "backslash" => Ok(Key::Layout('\\')),
        "bar" => Ok(Key::Layout('|')),
        "braceleft" => Ok(Key::Layout('{')),
        "braceright" => Ok(Key::Layout('}')),
        "bracketleft" => Ok(Key::Layout('[')),
        "bracketright" => Ok(Key::Layout(']')),
        "brokenbar" => Ok(Key::Layout('¦')),
        "ccedilla" => Ok(Key::Layout('ç')),
        "cedilla" => Ok(Key::Layout('¸')),
        "cent" => Ok(Key::Layout('¢')),
        "clear" => Ok(Key::Layout('\u{000b}')),
        "colon" => Ok(Key::Layout(':')),
        "comma" => Ok(Key::Layout(',')),
        "copyright" => Ok(Key::Layout('©')),
        "currency" => Ok(Key::Layout('¤')),
        "degree" => Ok(Key::Layout('°')),
        "diaeresis" => Ok(Key::Layout('¨')),
        "division" => Ok(Key::Layout('÷')),
        "dollar" => Ok(Key::Layout('$')),
        "eacute" => Ok(Key::Layout('é')),
        "ecircumflex" => Ok(Key::Layout('ê')),
        "ediaeresis" => Ok(Key::Layout('ë')),
        "egrave" => Ok(Key::Layout('è')),
        "equal" => Ok(Key::Layout('=')),
        "eth" => Ok(Key::Layout('ð')),
        "exclam" => Ok(Key::Layout('!')),
        "exclamdown" => Ok(Key::Layout('¡')),
        "grave" => Ok(Key::Layout('`')),
        "greater" => Ok(Key::Layout('>')),
        "guillemotleft" => Ok(Key::Layout('«')),
        "guillemotright" => Ok(Key::Layout('»')),
        "hyphen" => Ok(Key::Layout('­')),
        "iacute" => Ok(Key::Layout('í')),
        "icircumflex" => Ok(Key::Layout('î')),
        "idiaeresis" => Ok(Key::Layout('ï')),
        "igrave" => Ok(Key::Layout('ì')),
        "less" => Ok(Key::Layout('<')),
        "macron" => Ok(Key::Layout('¯')),
        "masculine" => Ok(Key::Layout('º')),
        "minus" => Ok(Key::Layout('-')),
        "mu" => Ok(Key::Layout('µ')),
        "multiply" => Ok(Key::Layout('×')),
        "nobreakspace" => Ok(Key::Layout('\u{00a0}')),
        "notsign" => Ok(Key::Layout('¬')),
        "ntilde" => Ok(Key::Layout('ñ')),
        "numbersign" => Ok(Key::Layout('#')),
        "oacute" => Ok(Key::Layout('ó')),
        "ocircumflex" => Ok(Key::Layout('ô')),
        "odiaeresis" => Ok(Key::Layout('ö')),
        "ograve" => Ok(Key::Layout('ò')),
        "onehalf" => Ok(Key::Layout('½')),
        "onequarter" => Ok(Key::Layout('¼')),
        "onesuperior" => Ok(Key::Layout('¹')),
        "ooblique" => Ok(Key::Layout('Ø')),
        "ordfeminine" => Ok(Key::Layout('ª')),
        "oslash" => Ok(Key::Layout('ø')),
        "otilde" => Ok(Key::Layout('õ')),
        "paragraph" => Ok(Key::Layout('¶')),
        "parenleft" => Ok(Key::Layout('(')),
        "parenright" => Ok(Key::Layout(')')),
        "percent" => Ok(Key::Layout('%')),
        "period" => Ok(Key::Layout('.')),
        "periodcentered" => Ok(Key::Layout('·')),
        "plus" => Ok(Key::Layout('+')),
        "plusminus" => Ok(Key::Layout('±')),
        "question" => Ok(Key::Layout('?')),
        "questiondown" => Ok(Key::Layout('¿')),
        "quotedbl" => Ok(Key::Layout('"')),
        "quoteleft" => Ok(Key::Layout('`')),
        "quoteright" => Ok(Key::Layout('\'')),
        "registered" => Ok(Key::Layout('®')),
        "section" => Ok(Key::Layout('§')),
        "semicolon" => Ok(Key::Layout(';')),
        "slash" => Ok(Key::Layout('/')),
        "ssharp" => Ok(Key::Layout('ß')),
        "sterling" => Ok(Key::Layout('£')),
        "thorn" => Ok(Key::Layout('þ')),
        "threequarters" => Ok(Key::Layout('¾')),
        "threesuperior" => Ok(Key::Layout('³')),
        "twosuperior" => Ok(Key::Layout('²')),
        "uacute" => Ok(Key::Layout('ú')),
        "ucircumflex" => Ok(Key::Layout('û')),
        "udiaeresis" => Ok(Key::Layout('ü')),
        "ugrave" => Ok(Key::Layout('ù')),
        "underscore" => Ok(Key::Layout('_')),
        "yacute" => Ok(Key::Layout('ý')),
        "ydiaeresis" => Ok(Key::Layout('ÿ')),
        "yen" => Ok(Key::Layout('¥')),
        _k => Err(ConversionError::UnknownKey(_k.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_convert_keyboard_shortcut() {
        assert_eq!(
            convert_keyboard_shortcut("{#Tab}").unwrap(),
            json!({ "cmds": [{ "Keys": [{ "Special": "Tab" }, []] }] })
        );
        assert_eq!(
            convert_keyboard_shortcut("{^}{#Tab}").unwrap(),
            json!({
                "cmds": [{ "Keys": [{ "Special": "Tab" }, []] }],
                "suppress_space_before": true,
            })
        );
        assert_eq!(
            convert_keyboard_shortcut("{#Tab}{^}").unwrap(),
            json!({
                "cmds": [{ "Keys": [{ "Special": "Tab" }, []] }],
                "text_after": "{^}",
            })
        );
        assert_eq!(
            convert_keyboard_shortcut("{^}{#Tab}{^}{-|}").unwrap(),
            json!({
                "cmds": [{ "Keys": [{ "Special": "Tab" }, []] }],
                "text_after": "{^}{-|}",
                "suppress_space_before": true,
            })
        );
    }

    #[test]
    fn test_parse_key_combo() {
        assert_eq!(
            parse_key_combo("Control_L(Alt_L(Super_L(Left)))").unwrap(),
            Command::Keys(
                Key::Special(SpecialKey::LeftArrow),
                vec![Modifier::Control, Modifier::Alt, Modifier::Meta]
            )
        );
        assert_eq!(
            parse_key_combo("option(a)").unwrap(),
            Command::Keys(Key::Layout('a'), vec![Modifier::Option])
        );
        assert_eq!(
            parse_key_combo("bAcKsPacE").unwrap(),
            Command::Keys(Key::Special(SpecialKey::Backspace), vec![])
        );
    }

    #[test]
    fn test_keyboard_shortcut_fails() {
        assert_eq!(
            convert_keyboard_shortcut("{#Tab Tab}").unwrap_err(),
            ConversionError::InvalidFormat
        );
        assert_eq!(
            convert_keyboard_shortcut("{#super(a) super(b)}").unwrap_err(),
            ConversionError::InvalidFormat
        );
        assert_eq!(
            convert_keyboard_shortcut("{#shift_l(space space)}").unwrap_err(),
            ConversionError::InvalidFormat
        );
        assert_eq!(
            convert_keyboard_shortcut("{#shift_l(alt_l(b)}").unwrap_err(),
            ConversionError::UnbalancedParens
        );
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serialize(&json!({
                "T-R": "interest",
                "PAT": "pat",
                "H-L": "hello",
                "WORLD": "world",
                "R-R": {
                    "cmds": [{ "Keys": [{ "Special": "Tab" }, []] }],
                },
            })),
            r#"{
"H-L": "hello",
"PAT": "pat",
"R-R": {"cmds":[{"Keys":[{"Special":"Tab"},[]]}]},
"T-R": "interest",
"WORLD": "world"
}"#
            .to_string()
        );
        assert_eq!(
            serialize(&json!({
                "R-R": {
                    "cmds": [{ "Keys": [{ "Special": "Tab" }, []] }],
                    "suppress_space_before": true,
                    "text_after": "{^}{-|}",
                },
            })),
            r#"{
"R-R": {"cmds":[{"Keys":[{"Special":"Tab"},[]]}],"suppress_space_before":true,"text_after":"{^}{-|}"}
}"#
            .to_string()
        );
        assert_eq!(serialize(&json!({})), "{}".to_string());
    }
}
