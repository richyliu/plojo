//! Controls the keyboard and mouse. Currently this is only a wrapper over enigo with support for
//! controlling the timing of key presses.

use std::{thread, time::Duration};

use enigo::KeyboardControllable;
use enigo::{Enigo, Key};
use translator::{Command, Key as InternalKey};

pub struct Controller {
    enigo: Enigo,
}

// Delay between pressing backspace (for corrections)
const BACKSPACE_DELAY: u32 = 2;
// Delay between pressing keys for typing normal text
const KEY_DELAY: u32 = 5;
// Delay between starting to hold down keys for keyboard shortcuts
const KEY_HOLD_DELAY: u32 = 10;

impl Controller {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(),
        }
    }

    pub fn dispatch(&mut self, command: Command) {
        match command {
            Command::Replace(backspace_num, add_text) => {
                if backspace_num > 0 {
                    self.backspace(backspace_num, BACKSPACE_DELAY);
                }

                if add_text.len() > 0 {
                    self.type_with_delay(&add_text, KEY_DELAY);
                }
            }
            Command::PrintHello => {
                println!("Hello!");
            }
            Command::NoOp => {}
            Command::Keys(keys) => self.key_combo(
                keys.iter().map(|k| from_internal_key(k)).collect(),
                KEY_HOLD_DELAY,
            ),
        }
    }

    fn type_with_delay(&mut self, text: &str, delay: u32) {
        let duration = Duration::from_millis(delay.into());
        for c in text.chars() {
            self.enigo.key_sequence(&c.to_string());
            thread::sleep(duration);
        }
    }

    /// Press the backspace key with specified delay in milliseconds between each press
    fn backspace(&mut self, num: usize, delay: u32) {
        let duration = Duration::from_millis(delay.into());
        for _ in 0..num {
            self.enigo.key_click(Key::Backspace);
            thread::sleep(duration);
        }
    }

    fn key_combo(&mut self, keys: Vec<Key>, delay: u32) {
        let duration = Duration::from_millis(delay.into());
        for k in &keys {
            self.enigo.key_down(*k);
            thread::sleep(duration);
        }

        for k in &keys {
            self.enigo.key_up(*k);
        }
    }
}

fn from_internal_key(key: &InternalKey) -> Key {
    match *key {
        InternalKey::Alt => Key::Alt,
        InternalKey::Backspace => Key::Backspace,
        InternalKey::CapsLock => Key::CapsLock,
        InternalKey::Control => Key::Control,
        InternalKey::Delete => Key::Delete,
        InternalKey::DownArrow => Key::DownArrow,
        InternalKey::End => Key::End,
        InternalKey::Escape => Key::Escape,
        InternalKey::F1 => Key::F1,
        InternalKey::F10 => Key::F10,
        InternalKey::F11 => Key::F11,
        InternalKey::F12 => Key::F12,
        InternalKey::F2 => Key::F2,
        InternalKey::F3 => Key::F3,
        InternalKey::F4 => Key::F4,
        InternalKey::F5 => Key::F5,
        InternalKey::F6 => Key::F6,
        InternalKey::F7 => Key::F7,
        InternalKey::F8 => Key::F8,
        InternalKey::F9 => Key::F9,
        InternalKey::Home => Key::Home,
        InternalKey::LeftArrow => Key::LeftArrow,
        InternalKey::Meta => Key::Meta,
        InternalKey::Option => Key::Option,
        InternalKey::PageDown => Key::PageDown,
        InternalKey::PageUp => Key::PageUp,
        InternalKey::Return => Key::Return,
        InternalKey::RightArrow => Key::RightArrow,
        InternalKey::Shift => Key::Shift,
        InternalKey::Space => Key::Space,
        InternalKey::Tab => Key::Tab,
        InternalKey::UpArrow => Key::UpArrow,
        InternalKey::Layout(c) => Key::Layout(c),
        InternalKey::Raw(raw) => Key::Raw(raw),
    }
}
