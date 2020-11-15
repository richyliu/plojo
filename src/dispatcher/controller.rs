//! Controls the keyboard and mouse. Currently this is only a wrapper over enigo with support for
//! controlling the timing of key presses.

use std::{thread, time::Duration};

use enigo::KeyboardControllable;
use enigo::MouseControllable;
use enigo::{Enigo, Key};

pub struct Controller {
    enigo: Enigo,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            enigo: Enigo::new(),
        }
    }

    pub fn type_no_delay(&mut self, text: &str) {
        self.enigo.key_sequence(text);
    }

    pub fn type_with_delay(&mut self, text: &str, delay: u32) {
        let duration = Duration::from_millis(delay.into());
        for c in text.chars() {
            self.enigo.key_sequence(&c.to_string());
            thread::sleep(duration);
        }
    }

    /// Press the backspace key with specified delay in milliseconds between each press
    pub fn backspace(&mut self, num: usize, delay: u32) {
        let duration = Duration::from_millis(delay.into());
        for _ in 0..num {
            self.enigo.key_click(Key::Backspace);
            thread::sleep(duration);
        }
    }

    pub fn mouse_move_to(&mut self, x: i32, y: i32) {
        self.enigo.mouse_move_to(x, y);
    }
}
