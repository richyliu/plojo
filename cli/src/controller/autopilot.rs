//! Dispatch commands with autopilot crate. Currently keyboard shortcut commands are not supported

use super::Controller;
use autopilot::key;
use autopilot::key::{Code, KeyCode};
use plojo_core::Command;
use std::process::Command as ProcessCommand;

const BACKSPACE_DELAY: u64 = 5;
const TYPE_SPEED: f64 = 400.0;

pub struct AutopilotController {}

fn dispatch_shell(cmd: String, args: Vec<String>) {
    let result = ProcessCommand::new(cmd).args(args).spawn();
    match result {
        Ok(_) => {}
        Err(e) => eprintln!("Could not execute shell command: {}", e),
    }
}

impl Controller for AutopilotController {
    fn new() -> Self {
        Self {}
    }

    fn dispatch(&mut self, command: Command) {
        match command {
            Command::Replace(backspace_num, add_text) => {
                for _ in 0..backspace_num {
                    key::tap(&Code(KeyCode::Backspace), &[], BACKSPACE_DELAY, 0);
                }

                if add_text.len() > 0 {
                    key::type_string(&add_text, &[], TYPE_SPEED, 0.);
                }
            }
            Command::PrintHello => {
                println!("Hello!");
            }
            Command::NoOp => {}
            Command::Keys(key, modifiers) => {
                eprintln!("Warning: autopilot controller does not support dispatching keys");
                eprintln!(
                    "Unable to dispatch: {:?} with modifiers: {:?}",
                    key, modifiers
                );
            }
            Command::Raw(key) => {
                eprintln!("Warning: autopilot controller does not support dispatching raw keys");
                eprintln!("Unable to dispatch key code: {:?}", key);
            }
            Command::Shell(cmd, args) => dispatch_shell(cmd, args),
        }
    }
}
