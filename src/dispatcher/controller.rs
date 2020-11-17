//! Controls the keyboard and mouse. Currently this is only a wrapper over enigo with support for
//! controlling the timing of key presses.

use std::{thread, time::Duration};

use enigo::KeyboardControllable;
use enigo::{Enigo, Key};

pub struct Controller {
    enigo: Enigo,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(),
        }
    }

    pub fn parse(&mut self, actions: Vec<ControllerAction>) {
        for action in actions {
            match action {
                ControllerAction::TypeWithDelay(text, delay) => self.type_with_delay(&text, delay),
                ControllerAction::BackspaceWithDelay(num, delay) => self.backspace(num, delay),
            }
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

#[derive(Debug, PartialEq)]
pub enum ControllerAction {
    TypeWithDelay(String, u32),
    BackspaceWithDelay(usize, u32),
}
