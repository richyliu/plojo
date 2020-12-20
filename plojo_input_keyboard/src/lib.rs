#[macro_use]
extern crate lazy_static;

use plojo_core::{Machine, Stroke};
use rdev::{Event, EventType};
use std::{
    collections::HashSet,
    error::Error,
    hash::Hash,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

#[derive(Debug, PartialEq, Eq, Hash)]
struct Key(String);

impl Key {
    fn new(key: rdev::Key) -> Self {
        Self(format!("{:?}", key))
    }
}

/// Listen to the keyboard as a steno machine
///
/// Only 1 keyboard machine should be created at a time.
pub struct KeyboardMachine {
    down_keys: HashSet<Key>,
    up_keys: HashSet<Key>,
    stroke: Option<Stroke>,
}

impl KeyboardMachine {
    pub fn new() -> Self {
        Self {
            down_keys: HashSet::new(),
            up_keys: HashSet::new(),
            stroke: None,
        }
    }

    /// Handles a key pressed down or up
    fn handle_key(&mut self, key: Key, is_down: bool) {
        if is_down {
            self.down_keys.insert(key);
        } else {
            if self.down_keys.contains(&key) {
                self.down_keys.remove(&key);
            }
            self.up_keys.insert(key);

            // this stroke has ended once all the keys are up
            if self.down_keys.is_empty() {
                if self.stroke.is_some() {
                    panic!("received new stroke but old stroke has not been processed");
                }
                let stroke = convert_stroke(&Layout::steno_querty(), &self.up_keys);
                self.stroke = stroke;
                self.up_keys.clear();
            }
        }
    }

    /// Returns the stroke that has been formed or None if the stroke is not ready yet.
    /// This moves the stroke out of the machine.
    fn get_stroke(&mut self) -> Option<Stroke> {
        self.stroke.take()
    }
}

/// A mapping from hardware keys to chars to build a stroke
struct Layout {
    // left hand keys
    pub left_keys: Vec<(Key, char)>,
    // if no center key is present and there are right_keys, a dash (`-`) is inserted
    pub center_keys: Vec<(Key, char)>,
    // right hand keys
    pub right_keys: Vec<(Key, char)>,
}

impl Layout {
    fn steno_querty() -> Self {
        Self {
            left_keys: vec![
                (Key::new(rdev::Key::KeyQ), 'S'),
                (Key::new(rdev::Key::KeyA), 'S'),
                (Key::new(rdev::Key::KeyW), 'T'),
                (Key::new(rdev::Key::KeyS), 'K'),
                (Key::new(rdev::Key::KeyE), 'P'),
                (Key::new(rdev::Key::KeyD), 'W'),
                (Key::new(rdev::Key::KeyR), 'H'),
                (Key::new(rdev::Key::KeyF), 'R'),
            ],
            center_keys: vec![
                (Key::new(rdev::Key::KeyC), 'A'),
                (Key::new(rdev::Key::KeyV), 'O'),
                (Key::new(rdev::Key::KeyT), '*'),
                (Key::new(rdev::Key::KeyG), '*'),
                (Key::new(rdev::Key::KeyY), '*'),
                (Key::new(rdev::Key::KeyH), '*'),
                (Key::new(rdev::Key::KeyN), 'E'),
                (Key::new(rdev::Key::KeyM), 'U'),
            ],
            right_keys: vec![
                (Key::new(rdev::Key::KeyU), 'F'),
                (Key::new(rdev::Key::KeyJ), 'R'),
                (Key::new(rdev::Key::KeyI), 'P'),
                (Key::new(rdev::Key::KeyK), 'B'),
                (Key::new(rdev::Key::KeyO), 'L'),
                (Key::new(rdev::Key::KeyL), 'G'),
                (Key::new(rdev::Key::KeyP), 'T'),
                (Key::new(rdev::Key::SemiColon), 'S'),
                (Key::new(rdev::Key::LeftBracket), 'D'),
                (Key::new(rdev::Key::Quote), 'Z'),
            ],
        }
    }
}

/// Converts pressed keys to a stroke based on the layout. Returns None if none of the keys
/// pressed could be mapped to a stroke key
fn convert_stroke(layout: &Layout, keys: &HashSet<Key>) -> Option<Stroke> {
    // TODO: handle number strokes

    let mut left = String::new();
    // check each key in the layout to see it exists
    for (k, c) in &layout.left_keys {
        // add it to the string, but don't add it twice
        if keys.contains(k) && !left.contains(*c) {
            left.push(*c);
        }
    }

    // same thing for center
    let mut center = String::new();
    for (k, c) in &layout.center_keys {
        if keys.contains(k) && !center.contains(*c) {
            center.push(*c);
        }
    }

    // same thing for right
    let mut right = String::new();
    for (k, c) in &layout.right_keys {
        if keys.contains(k) && !right.contains(*c) {
            right.push(*c);
        }
    }

    // put a dash if there is no center and there are right keys
    if center.is_empty() && !right.is_empty() {
        center = "-".to_string();
    }

    let result = left + &center + &right;
    if result.is_empty() {
        None
    } else {
        Some(Stroke::new(&result))
    }
}

lazy_static! {
    // Pass messages between the event handler and the keyboard machine
    static ref PASSER: (
        Arc<Mutex<Sender<(Key, bool)>>>,
        Arc<Mutex<Receiver<(Key, bool)>>>
    ) = {
        // spawn the listener here so it's not duplicated
        std::thread::spawn(|| {
            if let Err(e) = rdev::grab(handle_event) {
                eprintln!("Error listening to system events: {:?}", e);
                panic!("couldn't listen to system events");
            }
        });

        let (sender, receiver) = mpsc::channel();
        (Arc::new(Mutex::new(sender)), Arc::new(Mutex::new(receiver)))
    };
}

impl Machine for KeyboardMachine {
    fn read(&mut self) -> Result<Stroke, Box<dyn Error>> {
        loop {
            let receiver = PASSER.1.lock().unwrap();
            // wait for the next key
            if let Ok((key, is_down)) = receiver.recv() {
                self.handle_key(key, is_down);
            }

            // if this key finished the stroke, return it
            if let Some(stroke) = self.get_stroke() {
                return Ok(stroke);
            }
        }
    }
}

/// Handle a native event
///
/// This is used in rdev::listen, which only takes a fn pointer, which forces me to use Arc<Mutex>
/// and lazy static.
fn handle_event(event: Event) -> Option<Event> {
    // TODO: this doesn't work because it picks up the event from its own dispatching
    let (key, is_down) = match event.event_type {
        EventType::KeyPress(key) => (key, true),
        EventType::KeyRelease(key) => (key, false),
        _ => {
            // ignore all other events
            return Some(event);
        }
    };

    let sender = PASSER.0.lock().unwrap();
    sender.send((Key::new(key), is_down)).unwrap();

    // suppress the event
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_stroke_basic() {
        fn convert(keys: Vec<rdev::Key>) -> Option<Stroke> {
            convert_stroke(
                &Layout::steno_querty(),
                &keys.into_iter().map(Key::new).collect::<HashSet<_>>(),
            )
        }

        assert_eq!(
            convert(vec![
                rdev::Key::KeyQ,
                rdev::Key::KeyA,
                rdev::Key::KeyT,
                rdev::Key::KeyG,
            ])
            .unwrap(),
            Stroke::new("S*")
        );
        assert_eq!(
            convert(vec![rdev::Key::KeyQ, rdev::Key::KeyC, rdev::Key::KeyU]).unwrap(),
            Stroke::new("SAF")
        );
        assert_eq!(
            convert(vec![
                rdev::Key::KeyZ,
                rdev::Key::KeyQ,
                rdev::Key::KeyC,
                rdev::Key::KeyU
            ])
            .unwrap(),
            Stroke::new("SAF")
        );
        assert!(convert(vec![rdev::Key::KeyZ]).is_none());
    }

    #[test]
    fn handle_key_basic() {
        let mut m = KeyboardMachine::new();
        m.handle_key(Key::new(rdev::Key::KeyQ), true);
        assert!(m.get_stroke().is_none());
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        assert!(m.get_stroke().is_none());
        m.handle_key(Key::new(rdev::Key::KeyQ), false);
        assert!(m.get_stroke().is_none());
        m.handle_key(Key::new(rdev::Key::KeyW), false);

        assert_eq!(m.get_stroke().unwrap(), Stroke::new("ST"));
    }

    #[test]
    fn handle_key_mixed_order() {
        let mut m = KeyboardMachine::new();
        m.handle_key(Key::new(rdev::Key::KeyQ), true);
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        m.handle_key(Key::new(rdev::Key::KeyI), true);
        m.handle_key(Key::new(rdev::Key::KeyI), false);
        m.handle_key(Key::new(rdev::Key::KeyQ), false);
        m.handle_key(Key::new(rdev::Key::KeyW), false);

        assert_eq!(m.get_stroke().unwrap(), Stroke::new("ST-P"));
    }

    #[test]
    fn handle_key_multiple_presses() {
        let mut m = KeyboardMachine::new();
        m.handle_key(Key::new(rdev::Key::KeyQ), true);
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        m.handle_key(Key::new(rdev::Key::KeyW), false);
        m.handle_key(Key::new(rdev::Key::KeyQ), false);

        assert_eq!(m.get_stroke().unwrap(), Stroke::new("ST"));
    }

    #[test]
    fn handle_key_ignore_other_keys() {
        let mut m = KeyboardMachine::new();
        m.handle_key(Key::new(rdev::Key::KeyQ), true);
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        m.handle_key(Key::new(rdev::Key::BackSlash), true);
        m.handle_key(Key::new(rdev::Key::KeyW), false);
        m.handle_key(Key::new(rdev::Key::KeyQ), false);
        m.handle_key(Key::new(rdev::Key::BackSlash), false);

        assert_eq!(m.get_stroke().unwrap(), Stroke::new("ST"));
    }

    #[test]
    fn handle_key_multiple_strokes() {
        let mut m = KeyboardMachine::new();
        m.handle_key(Key::new(rdev::Key::KeyQ), true);
        m.handle_key(Key::new(rdev::Key::KeyW), true);
        m.handle_key(Key::new(rdev::Key::KeyW), false);
        m.handle_key(Key::new(rdev::Key::KeyQ), false);
        assert_eq!(m.get_stroke().unwrap(), Stroke::new("ST"));

        m.handle_key(Key::new(rdev::Key::KeyU), true);
        m.handle_key(Key::new(rdev::Key::KeyI), true);
        m.handle_key(Key::new(rdev::Key::KeyI), false);
        m.handle_key(Key::new(rdev::Key::KeyU), false);
        assert_eq!(m.get_stroke().unwrap(), Stroke::new("-FP"));
    }
}