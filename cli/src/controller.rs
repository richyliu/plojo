//! Controls the keyboard and mouse. Currently this is only a wrapper over enigo with support for
//! controlling the timing of key presses.

use std::{thread, time::Duration};

use enigo::KeyboardControllable;
use enigo::{Enigo, Key};
use translator::Command;

pub struct Controller {
    enigo: Enigo,
}

const BACKSPACE_DELAY: u32 = 2;
const KEY_DELAY: u32 = 5;

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
}
