use enigo::KeyboardControllable;
use enigo::{Enigo, Key};
use plojo_core::{Command, Controller, Key as InternalKey, Modifier, SpecialKey};
use std::{process::Command as ProcessCommand, thread, time::Duration};

pub struct EnigoController {
    enigo: Enigo,
}

// NOTE: these are irrelevant because enigo imposes a delay of 20 milliseconds for every key press
// Delay between pressing backspace (for corrections)
const BACKSPACE_DELAY: u64 = 2;
// Delay between pressing keys for typing normal text
const KEY_DELAY: u64 = 5;
// Delay between starting to hold down keys for keyboard shortcuts
const KEY_HOLD_DELAY: u64 = 2;

impl EnigoController {
    fn type_with_delay(&mut self, text: &str, delay: u64) {
        for c in text.chars() {
            self.enigo.key_sequence(&c.to_string());
            thread::sleep(Duration::from_millis(delay));
        }
    }

    /// Press the backspace key with specified delay in milliseconds between each press
    fn backspace(&mut self, num: usize, delay: u64) {
        for _ in 0..num {
            self.enigo.key_click(Key::Backspace);
            thread::sleep(Duration::from_millis(delay));
        }
    }

    fn key_combo(&mut self, keys: Vec<Key>, delay: u64) {
        for k in &keys {
            self.enigo.key_down(*k);
            thread::sleep(Duration::from_millis(delay));
        }

        for k in &keys {
            self.enigo.key_up(*k);
        }
    }
}

impl Controller for EnigoController {
    fn new(_disable_scan_keymap: bool) -> Self {
        // enigo does not scan keymap, so ignore the option
        Self {
            enigo: Enigo::new(),
        }
    }

    fn dispatch(&mut self, command: Command) {
        match command {
            Command::Replace(backspace_num, add_text) => {
                if backspace_num > 0 {
                    self.backspace(backspace_num, BACKSPACE_DELAY);
                }

                if !add_text.is_empty() {
                    self.type_with_delay(&add_text, KEY_DELAY);
                }
            }
            Command::PrintHello => {
                println!("Hello!");
            }
            Command::NoOp => {}
            Command::Keys(key, modifiers) => {
                let mut keys = Vec::with_capacity(modifiers.len() + 1);
                for m in modifiers {
                    keys.push(from_modifier(m));
                }
                keys.push(from_internal_key(key));
                self.key_combo(keys, KEY_HOLD_DELAY);
            }
            Command::Raw(code) => {
                self.enigo.key_click(Key::Raw(code));
            }
            Command::Shell(cmd, args) => dispatch_shell(cmd, args),
            Command::TranslatorCommand(_) => panic!("cannot handle translator command"),
        }
    }
}

fn from_internal_key(key: InternalKey) -> Key {
    match key {
        InternalKey::Special(special_key) => match special_key {
            SpecialKey::Backspace => Key::Backspace,
            SpecialKey::CapsLock => Key::CapsLock,
            SpecialKey::Delete => Key::Delete,
            SpecialKey::DownArrow => Key::DownArrow,
            SpecialKey::End => Key::End,
            SpecialKey::Escape => Key::Escape,
            SpecialKey::F1 => Key::F1,
            SpecialKey::F10 => Key::F10,
            SpecialKey::F11 => Key::F11,
            SpecialKey::F12 => Key::F12,
            SpecialKey::F2 => Key::F2,
            SpecialKey::F3 => Key::F3,
            SpecialKey::F4 => Key::F4,
            SpecialKey::F5 => Key::F5,
            SpecialKey::F6 => Key::F6,
            SpecialKey::F7 => Key::F7,
            SpecialKey::F8 => Key::F8,
            SpecialKey::F9 => Key::F9,
            SpecialKey::Home => Key::Home,
            SpecialKey::LeftArrow => Key::LeftArrow,
            SpecialKey::PageDown => Key::PageDown,
            SpecialKey::PageUp => Key::PageUp,
            SpecialKey::Return => Key::Return,
            SpecialKey::RightArrow => Key::RightArrow,
            SpecialKey::Space => Key::Space,
            SpecialKey::Tab => Key::Tab,
            SpecialKey::UpArrow => Key::Raw(0x7e), // NOTE: fixes a bug in enigo
        },
        InternalKey::Layout(c) => Key::Layout(c),
    }
}

fn from_modifier(modifier: Modifier) -> Key {
    match modifier {
        Modifier::Alt => Key::Alt,
        Modifier::Control => Key::Control,
        Modifier::Meta => Key::Meta,
        Modifier::Option => Key::Option,
        Modifier::Shift => Key::Shift,
    }
}

fn dispatch_shell(cmd: String, args: Vec<String>) {
    let result = ProcessCommand::new(cmd).args(args).spawn();
    match result {
        Ok(_) => {}
        Err(e) => eprintln!("[WARN] Could not execute shell command: {}", e),
    }
}
