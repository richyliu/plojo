/// What action should be taken
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize)]
pub enum Command {
    /// Press backspace a certain number of times and type the string
    Replace(usize, String),
    PrintHello,
    NoOp,
    /// Press and hold down the keys in order, then releas them all at once
    Keys(Vec<Key>),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize)]
pub enum Key {
    Alt,
    Backspace,
    CapsLock,
    Control,
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
    Meta,
    Option,
    PageDown,
    PageUp,
    Return,
    RightArrow,
    Shift,
    Space,
    Tab,
    UpArrow,
    Layout(char),
    Raw(u16),
}

impl Command {
    pub fn add_text(output: &str) -> Self {
        Self::replace_text(0, output)
    }
    pub fn replace_text(backspace_num: usize, replace_str: &str) -> Self {
        Self::Replace(backspace_num, replace_str.to_owned())
    }
}
