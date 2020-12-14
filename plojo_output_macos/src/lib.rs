//! Dispatch commands natively using core graphics and core foundations.

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode, KeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use plojo_core::{Command, Controller, Key, Modifier, SpecialKey};
use std::{collections::HashMap, process, thread, time::Duration};

// How long a key is held down
const KEY_HOLD_DELAY: u64 = 2;
// Delay between successive backspaces for corrections
const BACKSPACE_DELAY: u64 = 2;
// Delay between successive letters for typing normal text
const TYPE_DELAY: u64 = 5;
// Delay for holding down each modifier key
const MODIFIER_DELAY: u64 = 2;

pub struct MacController {}

impl Controller for MacController {
    fn new() -> Self {
        Self {}
    }

    fn dispatch(&mut self, command: Command) {
        // build a new map on each dispatch in case the keyboard layout changed
        // this map converts chars to keycodes in a keyboard shortcut
        let keycode_map = build_char_to_keycode_map();

        match command {
            Command::Replace(backspace_num, add_text) => {
                // tap backspace for corrections
                for _ in 0..backspace_num {
                    toggle_key(KeyCode::DELETE, true, &[], MODIFIER_DELAY);
                    thread::sleep(Duration::from_millis(KEY_HOLD_DELAY));
                    toggle_key(KeyCode::DELETE, false, &[], MODIFIER_DELAY);
                    thread::sleep(Duration::from_millis(BACKSPACE_DELAY));
                }

                // type text
                if add_text.len() > 0 {
                    for c in add_text.chars() {
                        type_char(c, true);
                        thread::sleep(Duration::from_millis(KEY_HOLD_DELAY));
                        type_char(c, false);
                        thread::sleep(Duration::from_millis(TYPE_DELAY));
                    }
                }
            }
            Command::PrintHello => {
                println!("Hello!");
            }
            Command::NoOp => {}
            Command::Keys(key, modifiers) => {
                let keycode = match key {
                    Key::Layout(c) => {
                        // try to convert the char to a physical key
                        if let Some(code) = keycode_map.get(&c) {
                            *code
                        } else {
                            eprintln!(
                                "Cannot do keyboard shortcut of {:?} with {:?}",
                                key, modifiers
                            );
                            eprintln!("Cannot convert the key to a physical key");
                            return;
                        }
                    }
                    Key::Special(special_key) => key_to_keycode(special_key),
                };
                toggle_key(keycode, true, &modifiers, MODIFIER_DELAY);
                thread::sleep(Duration::from_millis(KEY_HOLD_DELAY));
                toggle_key(keycode, false, &modifiers, MODIFIER_DELAY);
            }
            Command::Raw(key) => {
                toggle_key(key, true, &[], MODIFIER_DELAY);
                thread::sleep(Duration::from_millis(KEY_HOLD_DELAY));
                toggle_key(key, false, &[], MODIFIER_DELAY);
            }
            Command::Shell(cmd, args) => dispatch_shell(cmd, args),
        }
    }
}

fn dispatch_shell(cmd: String, args: Vec<String>) {
    let result = process::Command::new(cmd).args(args).spawn();
    match result {
        Ok(_) => {}
        Err(e) => eprintln!("Could not execute shell command: {}", e),
    }
}

/// Types a single char. Supports UTF-8
fn type_char(c: char, down: bool) {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
    let event = CGEvent::new_keyboard_event(source, 0, down).unwrap();
    let mut buf = [0; 2];
    event.set_string_from_utf16_unchecked(c.encode_utf16(&mut buf));
    event.post(CGEventTapLocation::HID);
}

/// Toggles a physical key with support for modifiers
///
/// Key down and key up for modifiers must be handled differently. This is the only way to prevent
/// glitches in the modifier not being detected
fn toggle_key(key: CGKeyCode, down: bool, modifiers: &[Modifier], modifier_delay: u64) {
    // key down must be triggered with modifiers as flags...
    if down {
        if modifiers == &[Modifier::Control] {
            handle_keydown_control_and_arrow(key, modifier_delay);
        }
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
        let event = CGEvent::new_keyboard_event(source, key, true).unwrap();
        event.set_flags(modifiers_to_flags(modifiers));
        event.post(CGEventTapLocation::HID);
    } else {
        // ... while keyup must release the modifiers individually as keys
        for m in modifiers {
            let modifier_key = modifier_to_key(*m);
            let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
            let event = CGEvent::new_keyboard_event(source, modifier_key, false).unwrap();
            event.post(CGEventTapLocation::HID);
            thread::sleep(Duration::from_millis(modifier_delay));
        }
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
        let event = CGEvent::new_keyboard_event(source, key, false).unwrap();
        event.post(CGEventTapLocation::HID);
    }
}

/// If key is an arrow key, it must be handled differently.
///
/// We need to do this because of a bug in the Carbon API
fn handle_keydown_control_and_arrow(key: CGKeyCode, modifier_delay: u64) {
    match key {
        KeyCode::UP_ARROW | KeyCode::DOWN_ARROW | KeyCode::LEFT_ARROW | KeyCode::RIGHT_ARROW => {
            // press the control separately instead of as a flag
            let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
            let event = CGEvent::new_keyboard_event(source, KeyCode::CONTROL, true).unwrap();
            event.post(CGEventTapLocation::HID);

            thread::sleep(Duration::from_millis(modifier_delay));

            let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
            let event = CGEvent::new_keyboard_event(source, key, true).unwrap();
            event.post(CGEventTapLocation::HID);
        }
        _ => {}
    }
}

fn modifiers_to_flags(modifiers: &[Modifier]) -> CGEventFlags {
    let mut flags = CGEventFlags::CGEventFlagNull;
    for m in modifiers {
        flags |= match m {
            Modifier::Alt => CGEventFlags::CGEventFlagAlternate,
            Modifier::Control => CGEventFlags::CGEventFlagControl,
            Modifier::Meta => CGEventFlags::CGEventFlagCommand,
            Modifier::Option => CGEventFlags::CGEventFlagAlternate,
            Modifier::Shift => CGEventFlags::CGEventFlagShift,
        }
    }
    flags
}

fn modifier_to_key(modifier: Modifier) -> CGKeyCode {
    match modifier {
        Modifier::Alt => KeyCode::OPTION,
        Modifier::Control => KeyCode::CONTROL,
        Modifier::Meta => KeyCode::COMMAND,
        Modifier::Option => KeyCode::OPTION,
        Modifier::Shift => KeyCode::SHIFT,
    }
}

fn key_to_keycode(key: SpecialKey) -> CGKeyCode {
    match key {
        SpecialKey::Backspace => KeyCode::DELETE,
        SpecialKey::CapsLock => KeyCode::CAPS_LOCK,
        SpecialKey::Delete => KeyCode::FORWARD_DELETE,
        SpecialKey::DownArrow => KeyCode::DOWN_ARROW,
        SpecialKey::End => KeyCode::END,
        SpecialKey::Escape => KeyCode::ESCAPE,
        SpecialKey::F1 => KeyCode::F1,
        SpecialKey::F10 => KeyCode::F10,
        SpecialKey::F11 => KeyCode::F11,
        SpecialKey::F12 => KeyCode::F12,
        SpecialKey::F2 => KeyCode::F2,
        SpecialKey::F3 => KeyCode::F3,
        SpecialKey::F4 => KeyCode::F4,
        SpecialKey::F5 => KeyCode::F5,
        SpecialKey::F6 => KeyCode::F6,
        SpecialKey::F7 => KeyCode::F7,
        SpecialKey::F8 => KeyCode::F8,
        SpecialKey::F9 => KeyCode::F9,
        SpecialKey::Home => KeyCode::HOME,
        SpecialKey::LeftArrow => KeyCode::LEFT_ARROW,
        SpecialKey::PageDown => KeyCode::PAGE_DOWN,
        SpecialKey::PageUp => KeyCode::PAGE_UP,
        SpecialKey::Return => KeyCode::RETURN,
        SpecialKey::RightArrow => KeyCode::RIGHT_ARROW,
        SpecialKey::Space => KeyCode::SPACE,
        SpecialKey::Tab => KeyCode::TAB,
        SpecialKey::UpArrow => KeyCode::UP_ARROW,
    }
}

/// Build a hashmap between the letter and its physical key (layout dependent)
fn build_char_to_keycode_map() -> HashMap<char, CGKeyCode> {
    let mut map = HashMap::new();
    // check each key code to see if it represents a char
    for i in 0..128 {
        if let Some(c) = keycode_to_char(i) {
            map.insert(c, i);
        }
    }
    map
}

fn keycode_to_char(code: CGKeyCode) -> Option<char> {
    use cocoa::appkit::{NSEvent, NSEventType};
    use cocoa::foundation::NSString;
    use foreign_types::ForeignType;
    use std::{slice, str};

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
    let event = CGEvent::new_keyboard_event(source, code, true).unwrap();

    unsafe {
        let ns_event = NSEvent::eventWithCGEvent_(
            cocoa::appkit::NSGeneralPboard, // this can be anything; it isn't actually used
            event.as_ptr() as *mut core::ffi::c_void,
        );

        if ns_event.eventType() == NSEventType::NSKeyDown {
            let chars = ns_event.characters();
            let str_ptr = slice::from_raw_parts(chars.UTF8String() as *const u8, chars.len());
            let string = str::from_utf8(str_ptr);

            if let Ok(s) = string {
                let mut chars = s.chars();
                if let Some(c) = chars.next() {
                    assert_eq!(chars.next(), None); // should be only 1 char
                    return Some(c);
                }
            }
        } else {
            // the key code didn't result in a character
        }
    }

    // couldn't convert the char
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycode_conversion() {
        // NOTE: if you hold down shift while running this test, it will fail
        // on QWERTY layout
        assert_eq!(keycode_to_char(0), Some('a'));
        assert_eq!(keycode_to_char(6), Some('z'));
        assert_eq!(keycode_to_char(53), Some('\u{1b}'));

        // control key
        assert_eq!(keycode_to_char(59), None);
    }
}
