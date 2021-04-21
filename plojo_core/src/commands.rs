/// What action should be taken
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum Command {
    /// Press backspace a certain number of times and type the string
    Replace(usize, String),
    PrintHello,
    NoOp,
    /// Press a key with some modifier keys
    Keys(Key, Vec<Modifier>),
    /// Send a raw keystroke with key code
    Raw(u16),
    /// Dispatch a shell command with arguments
    Shell(String, Vec<String>),
    /// Pass a command to the translator to be handled
    TranslatorCommand(String),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum Key {
    Special(SpecialKey),
    Layout(char), // literal key (ex: "a", "b", etc.)
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum SpecialKey {
    Backspace,
    CapsLock,
    Delete,
    DownArrow,
    End,
    Escape,
    F1,
    F10,
    F11,
    F12,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Home,
    LeftArrow,
    PageDown,
    PageUp,
    Return,
    RightArrow,
    Space,
    Tab,
    UpArrow,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, Copy)]
pub enum Modifier {
    Alt,
    Control,
    Meta,
    Option, // for MacOS
    Shift,
    Fn,
}

impl Command {
    pub fn add_text(output: &str) -> Self {
        Self::replace_text(0, output)
    }
    pub fn replace_text(backspace_num: usize, replace_str: &str) -> Self {
        Self::Replace(backspace_num, replace_str.to_owned())
    }
}
