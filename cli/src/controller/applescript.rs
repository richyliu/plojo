use super::Controller;
use plojo_core::{Command, Key, Modifier, SpecialKey};
use std::process::Command as ProcessCommand;

pub struct ApplescriptController {}

#[derive(Debug, PartialEq)]
struct Cmd(String);

/// Performs the keystroke command using oscscripts
///
/// Panics if the osascript command failed
fn osascript_cmd(cmd: Cmd) {
    let applescript_cmd = format!(r#"tell application "System Events" to {}"#, cmd.0);
    let status = ProcessCommand::new("osascript")
        .arg("-e")
        .arg(applescript_cmd)
        .status()
        .expect("Could not execute osascript for keystroke");

    if !status.success() {
        panic!("osascript non zero keycode for command: {}", cmd.0);
    }
}

fn type_string(text: &str) -> Cmd {
    let text = text.replace(r#"\"#, r#"\\"#); // escape backslashes
    let text = text.replace(r#"""#, r#"\""#); // escape quotes
    Cmd(format!(r#"keystroke "{}""#, text))
}

fn type_key(key: Key, modifiers: Vec<Modifier>) -> Cmd {
    let mut cmd_str = key_to_string(key);
    cmd_str.push_str(&match modifiers.len() {
        0 => String::new(),
        1 => format!(r#" using {{ {} down }}"#, modifier_to_string(modifiers[0])),
        _ => format!(
            r#" using {{ {} }}"#,
            modifiers
                .iter()
                .map(|m| modifier_to_string(*m) + " down")
                .collect::<Vec<_>>()
                .join(", ")
        ),
    });

    Cmd(cmd_str)
}

fn type_raw(code: u16) -> Cmd {
    Cmd(format!(r#"key code {}"#, code))
}

/// Trigger backspace n times using a loop in applescript
fn backspace(n: usize) {
    let status = ProcessCommand::new("osascript")
        .arg("-e")
        .arg(format!("repeat {} times", n))
        .arg("-e")
        // 51 is the key code for backspace
        .arg(r#"tell application "System Events" to key code 51"#)
        .arg("-e")
        .arg("end repeat")
        .status()
        .expect("Could not execute osascript for keystroke to press backspace");

    if !status.success() {
        panic!("osascript for backspace keystroke returned non zero keycode");
    }
}

fn dispatch_shell(cmd: String, args: Vec<String>) {
    let result = ProcessCommand::new(cmd).args(args).spawn();
    match result {
        Ok(_) => {}
        Err(e) => eprintln!("Could not execute shell command: {}", e),
    }
}

impl Controller for ApplescriptController {
    fn new() -> Self {
        Self {}
    }

    fn dispatch(&mut self, command: Command) {
        match command {
            Command::Replace(backspace_num, add_text) => {
                backspace(backspace_num);

                if add_text.len() > 0 {
                    osascript_cmd(type_string(&add_text));
                }
            }
            Command::PrintHello => {
                println!("Hello!");
            }
            Command::NoOp => {}
            Command::Keys(key, modifiers) => osascript_cmd(type_key(key, modifiers)),
            Command::Raw(code) => osascript_cmd(type_raw(code)),
            Command::Shell(cmd, args) => dispatch_shell(cmd, args),
        }
    }
}

fn key_to_string(key: Key) -> String {
    match key {
        // key code source: http://macbiblioblog.blogspot.com/2014/12/key-codes-for-function-and-special-keys.html
        Key::Special(special_key) => format!(
            r#"key code {}"#,
            match special_key {
                SpecialKey::Backspace => 51,
                SpecialKey::CapsLock => 57,
                SpecialKey::Delete => 117,
                SpecialKey::DownArrow => 125,
                SpecialKey::End => 119,
                SpecialKey::Escape => 53,
                SpecialKey::F1 => 122,
                SpecialKey::F10 => 109,
                SpecialKey::F11 => 103,
                SpecialKey::F12 => 111,
                SpecialKey::F2 => 120,
                SpecialKey::F3 => 99,
                SpecialKey::F4 => 118,
                SpecialKey::F5 => 96,
                SpecialKey::F6 => 97,
                SpecialKey::F7 => 98,
                SpecialKey::F8 => 100,
                SpecialKey::F9 => 101,
                SpecialKey::Home => 115,
                SpecialKey::LeftArrow => 123,
                SpecialKey::PageDown => 121,
                SpecialKey::PageUp => 116,
                SpecialKey::Return => 36, // KeypadEnter (code 76) might also be appropriate
                SpecialKey::RightArrow => 124,
                SpecialKey::Space => 49,
                SpecialKey::Tab => 48,
                SpecialKey::UpArrow => 126,
            }
        ),
        // use type_string to properly escape symbols
        Key::Layout(c) => type_string(&c.to_string()).0,
    }
}

fn modifier_to_string(modifier: Modifier) -> String {
    match modifier {
        Modifier::Alt => "option", // same as option key on mac
        Modifier::Control => "control",
        Modifier::Meta => "command",
        Modifier::Option => "option",
        Modifier::Shift => "shift",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_key() {
        assert_eq!(
            type_key(Key::Layout('a'), vec![Modifier::Meta]),
            Cmd(r#"keystroke "a" using { command down }"#.to_string())
        );
        assert_eq!(
            type_key(
                Key::Special(SpecialKey::PageUp),
                vec![Modifier::Control, Modifier::Shift]
            ),
            Cmd(r#"key code 116 using { control down, shift down }"#.to_string())
        );
        assert_eq!(
            type_key(Key::Special(SpecialKey::PageDown), vec![]),
            Cmd(r#"key code 121"#.to_string())
        );
    }

    #[test]
    fn test_escaping() {
        assert_eq!(type_string(r#" ""#), Cmd(r#"keystroke " \"""#.to_string()));
        assert_eq!(
            type_key(Key::Layout('\\'), vec![]),
            Cmd(r#"keystroke "\\""#.to_string())
        );
        assert_eq!(
            type_key(Key::Layout('"'), vec![]),
            Cmd(r#"keystroke "\"""#.to_string())
        );
    }
}